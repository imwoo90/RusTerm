//! # Chunk Handler & Stream Line Processor
//!
//! ## Overview
//! Coordinates incoming serial chunk buffering, VT100 terminal escape sequence emulation,
//! byte-boundary line splitting (`MAX_LINE_BYTES`), UTF-8 boundary backtracking, and hex dump line formatting.
//!
//! ## Search Tags
//! #chunk-handler, #vt100, #line-wrapping, #terminal-stream, #utf8-boundary, #hex-stream

#[cfg(test)]
#[path = "chunk_handler_tests.rs"]
mod tests;

use crate::config::MAX_LINE_BYTES;
use crate::worker::formatter::LogFormatterStrategy;
use crate::worker::repository::index::{ByteOffset, LineRange};
use ::vt100::Parser;
use std::borrow::Cow;

/// Cohesive accumulator tracking formatted line batches, relative byte offsets, and filter matches.
pub struct LineBatchAccumulator<'a, F: Fn(&str) -> bool> {
    /// Active formatting strategy (VT100/Text or Hex).
    pub formatter: &'a dyn LogFormatterStrategy,
    /// Optional formatted timestamp prefix (`[HH:MM:SS.mmm] `).
    pub timestamp: &'a str,
    /// Accumulated output string buffer for the current chunk.
    pub batch: String,
    /// Relative end byte offsets of each appended subline.
    pub offsets: Vec<ByteOffset>,
    /// Relative byte ranges of sublines matching the active filter.
    pub filtered: Vec<LineRange>,
    /// Running byte offset within the current batch.
    pub current_relative_offset: ByteOffset,
    /// Whether an active search/filter query is currently enabled.
    pub is_filtering: bool,
    /// Predicate evaluating whether a formatted subline matches the active filter.
    pub filter_matcher: F,
}

impl<'a, F: Fn(&str) -> bool> LineBatchAccumulator<'a, F> {
    /// Creates a new accumulator with pre-allocated batch capacity.
    pub fn with_capacity(
        capacity: usize,
        formatter: &'a dyn LogFormatterStrategy,
        timestamp: &'a str,
        is_filtering: bool,
        filter_matcher: F,
    ) -> Self {
        Self {
            formatter,
            timestamp,
            batch: String::with_capacity(capacity),
            offsets: Vec::new(),
            filtered: Vec::new(),
            current_relative_offset: ByteOffset(0),
            is_filtering,
            filter_matcher,
        }
    }

    /// Splits and formats a single logical line, appending to batch and offset trackers.
    pub fn append_line(&mut self, line: &str) {
        let max_len = self.formatter.max_line_length();
        let mut start = 0;

        if line.is_empty() {
            self.append_single_subline("");
            return;
        }

        while start < line.len() {
            let mut end = (start + max_len).min(line.len());
            while !line.is_char_boundary(end) {
                end -= 1;
            }
            let sub_line = &line[start..end];
            self.append_single_subline(sub_line);
            start = end;
        }
    }

    fn append_single_subline(&mut self, sub_line: &str) {
        let start_pos = self.batch.len();
        let formatted = self.formatter.format(sub_line, self.timestamp);
        self.batch.push_str(&formatted);
        let line_len = (self.batch.len() - start_pos) as u64;

        if self.is_filtering && (self.filter_matcher)(&self.batch[start_pos..]) {
            self.filtered.push(LineRange {
                start: self.current_relative_offset,
                end: self.current_relative_offset + line_len,
            });
        }

        self.current_relative_offset = self.current_relative_offset + line_len;
        self.offsets.push(self.current_relative_offset);
    }
}

/// Scans for next newline delimiter or reports buffer full boundary.
/// Returns `Some((content_end_index, next_start_index))` if found.
pub fn find_next_line_ending_or_full(
    chunk: &[u8],
    start: usize,
    remaining_space: usize,
) -> Option<(usize, usize)> {
    let len = chunk.len();
    let search_limit = std::cmp::min(start + remaining_space, len);

    let mut i = start;
    while i < search_limit {
        let b = chunk[i];
        if b == b'\n' {
            return Some((i, i + 1));
        } else if b == b'\r' {
            if i + 1 < len {
                if chunk[i + 1] == b'\n' {
                    return Some((i, i + 2)); // CRLF
                } else {
                    return Some((i, i + 1)); // CR
                }
            } else {
                // Trailing CR: buffer to next chunk
                return None;
            }
        }
        i += 1;
    }

    if search_limit == start + remaining_space {
        let cut = find_safe_utf8_cut(chunk, start, search_limit);
        return Some((cut, cut));
    }

    None
}

/// Backtracks up to 3 bytes if cut lands inside an incomplete UTF-8 multi-byte sequence.
fn find_safe_utf8_cut(chunk: &[u8], start: usize, search_limit: usize) -> usize {
    let mut cut_point = search_limit;
    match std::str::from_utf8(&chunk[start..cut_point]) {
        Ok(_) => cut_point,
        Err(e) => {
            let valid_len = e.valid_up_to();
            if start + valid_len >= cut_point.saturating_sub(3) {
                cut_point = start + valid_len;
            }
            cut_point
        }
    }
}

/// Extracts formatted row bytes from the VT100 parser screen.
pub fn extract_screen_row(parser: &Parser) -> Option<String> {
    parser
        .screen()
        .rows_formatted(0, MAX_LINE_BYTES as u16)
        .next()
        .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
}

/// Processes a VT100 byte chunk and generates batches, offsets, and active line.
pub fn process_vt100_stream(
    parser: &mut Parser,
    chunk: &[u8],
    formatter: &dyn LogFormatterStrategy,
    timestamp: &str,
    is_filtering: bool,
    filter_matcher: impl Fn(&str) -> bool,
) -> (String, Vec<ByteOffset>, Vec<LineRange>, Option<String>) {
    let mut acc = LineBatchAccumulator::with_capacity(
        0,
        formatter,
        timestamp,
        is_filtering,
        &filter_matcher,
    );

    let mut start = 0;
    let len = chunk.len();

    while start < len {
        let (_, col) = parser.screen().cursor_position();
        let remaining = MAX_LINE_BYTES - col as usize;

        if let Some((end, next_start)) = find_next_line_ending_or_full(chunk, start, remaining) {
            let line_bytes = &chunk[start..end];
            parser.process(line_bytes);

            if let Some(line_str) = extract_screen_row(parser) {
                acc.append_line(&line_str);
            }

            // Clear row for next line: Carriage Return + Clear Line
            parser.process(b"\r\x1b[2K");
            start = next_start;
        } else {
            break;
        }
    }

    if start < chunk.len() {
        parser.process(&chunk[start..]);
    }

    let active_line = extract_screen_row(parser)
        .filter(|s| !s.trim().is_empty())
        .filter(|s| !is_filtering || filter_matcher(s));

    (acc.batch, acc.offsets, acc.filtered, active_line)
}

/// Processes incoming hex string chunks, splitting by fixed max line length.
pub fn process_hex_stream(
    leftover_buffer: &mut String,
    chunk: &str,
    formatter: &dyn LogFormatterStrategy,
    timestamp: &str,
    is_filtering: bool,
    filter_matcher: impl Fn(&str) -> bool,
) -> (String, Vec<ByteOffset>, Vec<LineRange>, Option<String>) {
    let max_len = formatter.max_line_length();

    if !leftover_buffer.is_empty() && leftover_buffer.len() >= max_len {
        leftover_buffer.push('\n');
    }

    let full_text = if leftover_buffer.is_empty() {
        Cow::Borrowed(chunk)
    } else {
        Cow::Owned(format!("{}{}", leftover_buffer, chunk))
    };

    let mut acc = LineBatchAccumulator::with_capacity(
        full_text.len() * 2,
        formatter,
        timestamp,
        is_filtering,
        &filter_matcher,
    );

    let text_bytes = full_text.as_bytes();
    let len = text_bytes.len();
    let mut start = 0;

    while start < len {
        let remaining = len - start;
        if remaining >= max_len {
            let end = start + max_len;
            let line_str = &full_text[start..end];
            acc.append_line(line_str);
            start = end;
        } else {
            break;
        }
    }

    let active_line = if start < len {
        let leftover = full_text[start..].to_string();
        *leftover_buffer = leftover.clone();
        Some(leftover)
    } else {
        leftover_buffer.clear();
        None
    };

    (acc.batch, acc.offsets, acc.filtered, active_line)
}

/// Handles streaming line processing with leftover buffer management.
pub struct StreamingLineProcessor {
    pub leftover_buffer: String,
    parser: Parser,
}

impl StreamingLineProcessor {
    pub fn new() -> Self {
        Self {
            leftover_buffer: String::new(),
            // Height 1 ensures we focus on a single line.
            // Width MAX_LINE_BYTES prevents arbitrary wrapping of long lines.
            // Scrollback 0 disables history as we extract confirmed lines immediately.
            parser: Parser::new(1, MAX_LINE_BYTES as u16, 0),
        }
    }

    pub fn process_vt100(
        &mut self,
        chunk: &[u8],
        formatter: &dyn LogFormatterStrategy,
        timestamp: &str,
        is_filtering: bool,
        filter_matcher: impl Fn(&str) -> bool,
    ) -> (String, Vec<ByteOffset>, Vec<LineRange>, Option<String>) {
        process_vt100_stream(
            &mut self.parser,
            chunk,
            formatter,
            timestamp,
            is_filtering,
            filter_matcher,
        )
    }

    /// Processes a hex chunk (Hex mode)
    pub fn process_hex_lines(
        &mut self,
        chunk: &str,
        formatter: &dyn LogFormatterStrategy,
        timestamp: &str,
        is_filtering: bool,
        filter_matcher: impl Fn(&str) -> bool,
    ) -> (String, Vec<ByteOffset>, Vec<LineRange>, Option<String>) {
        process_hex_stream(
            &mut self.leftover_buffer,
            chunk,
            formatter,
            timestamp,
            is_filtering,
            filter_matcher,
        )
    }

    pub fn clear(&mut self) {
        self.leftover_buffer.clear();
        self.parser = Parser::new(1, MAX_LINE_BYTES as u16, 0);
    }
}

impl Default for StreamingLineProcessor {
    fn default() -> Self {
        Self::new()
    }
}
