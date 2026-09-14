//! # VT100 Stream Processor
//!
//! ## Overview
//! Processes raw byte chunks through an internal `vt100::Parser`, parses escape codes,
//! extracts completed lines, and tracks the current unfinished active line on screen.
//!
//! ## Search Tags
//! #vt100, #ansi-parser, #screen-row, #stream-processor

use crate::config::MAX_LINE_BYTES;
use crate::worker::chunk_handler::line_builder::format_and_append_line;
use crate::worker::chunk_handler::scanner::find_next_line_ending_or_full;
use crate::worker::formatter::LogFormatterStrategy;
use crate::worker::repository::index::{ByteOffset, LineRange};
use vt100::Parser;

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
    let mut batch = String::new();
    let mut offsets = Vec::new();
    let mut filtered = Vec::new();
    let mut relative_offset = ByteOffset(0);

    let mut start = 0;
    let len = chunk.len();

    while start < len {
        let (_, col) = parser.screen().cursor_position();
        let remaining = MAX_LINE_BYTES - col as usize;

        if let Some((end, next_start)) = find_next_line_ending_or_full(chunk, start, remaining) {
            let line_bytes = &chunk[start..end];
            parser.process(line_bytes);

            if let Some(line_str) = extract_screen_row(parser) {
                format_and_append_line(
                    &line_str,
                    formatter,
                    timestamp,
                    &mut batch,
                    &mut offsets,
                    &mut filtered,
                    &mut relative_offset,
                    is_filtering,
                    &filter_matcher,
                );
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

    (batch, offsets, filtered, active_line)
}
