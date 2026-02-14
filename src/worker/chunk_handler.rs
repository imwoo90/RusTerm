use crate::config::MAX_LINE_BYTES;
use crate::worker::formatter::LogFormatterStrategy;
use crate::worker::repository::index::{ByteOffset, LineRange};
use std::borrow::Cow;
use vt100::Parser;

/// Handles streaming line processing with leftover buffer management
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
        let mut batch = String::new();
        let mut offsets = Vec::new();
        let mut filtered = Vec::new();
        let mut relative_offset = ByteOffset(0);

        let mut start = 0;
        let len = chunk.len();

        while start < len {
            // Get current cursor position to determine remaining space on the line.
            // Since height is 1, row is always 0.
            let (_, col) = self.parser.screen().cursor_position();
            let remaining = MAX_LINE_BYTES - col as usize;

            if let Some((end, next_start)) =
                Self::find_next_line_ending_or_full(chunk, start, remaining)
            {
                // Process content up to the newline char(s) OR up to the full buffer limit
                let line_bytes = &chunk[start..end];
                self.parser.process(line_bytes);

                // Extract the formatted line immediately
                if let Some(bytes) = self
                    .parser
                    .screen()
                    .rows_formatted(0, MAX_LINE_BYTES as u16)
                    .next()
                {
                    let line_str = String::from_utf8_lossy(&bytes);

                    self.process_single_line(
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

                // Clear the line in the parser to prepare for the next line
                // Carriage Return + Clear Line
                self.parser.process(b"\r\x1b[2K");

                start = next_start;
            } else {
                // No more newlines AND buffer not full yet
                break;
            }
        }

        // Process any remaining bytes (incomplete line)
        if start < chunk.len() {
            self.parser.process(&chunk[start..]);
        }

        // Get Current Active Line (Row 0)
        // If the chunk ended with a newline, this will be empty (which is correct)
        let active_line = self
            .parser
            .screen()
            .rows_formatted(0, MAX_LINE_BYTES as u16)
            .next()
            .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
            .filter(|s| !s.trim().is_empty())
            .filter(|s| !is_filtering || filter_matcher(s));

        (batch, offsets, filtered, active_line)
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
        let max_len = formatter.max_line_length();

        // 1. If leftover is already too long, force a split before even adding new chunk
        if !self.leftover_buffer.is_empty() && self.leftover_buffer.len() >= max_len {
            self.leftover_buffer.push('\n');
        }

        let full_text = if self.leftover_buffer.is_empty() {
            Cow::Borrowed(chunk)
        } else {
            Cow::Owned(format!("{}{}", self.leftover_buffer, chunk))
        };

        let mut batch = String::with_capacity(full_text.len() * 2);
        let mut offsets = Vec::new(); // Capacity logic changed slightly, let vector handle realloc or use heuristic
        let mut filtered = Vec::new();
        let mut relative_offset = ByteOffset(0);

        let text_bytes = full_text.as_bytes();
        let len = text_bytes.len();
        let mut start = 0;

        while start < len {
            // Hex Mode: Fixed width splitting logic
            // We split strictly by max_len (e.g. 16 bytes = 48 chars).
            // No newline searching.
            let remaining = len - start;
            if remaining >= max_len {
                // If we have enough for a full line, extract it.
                let end = start + max_len;
                let line_str = &full_text[start..end];

                self.process_single_line(
                    line_str,
                    formatter,
                    timestamp,
                    &mut batch,
                    &mut offsets,
                    &mut filtered,
                    &mut relative_offset,
                    is_filtering,
                    &filter_matcher,
                );
                start = end;
            } else {
                // Not enough for a full line, buffer it.
                break;
            }
        }

        // Save remaining partial line
        let active_line = if start < len {
            let leftover = full_text[start..].to_string();
            self.leftover_buffer = leftover.clone();
            Some(leftover)
        } else {
            self.leftover_buffer.clear();
            None
        };

        // Return batch, offsets, filtered, and active_line (partial hex line)
        (batch, offsets, filtered, active_line)
    }

    #[allow(clippy::too_many_arguments)]
    fn process_single_line(
        &self,
        line: &str,
        formatter: &dyn LogFormatterStrategy,
        timestamp: &str,
        batch: &mut String,
        offsets: &mut Vec<ByteOffset>,
        filtered: &mut Vec<LineRange>,
        current_relative_offset: &mut ByteOffset,
        is_filtering: bool,
        filter_matcher: &impl Fn(&str) -> bool,
    ) {
        let max_len = formatter.max_line_length();
        let mut start = 0;

        // Handle empty line case
        if line.is_empty() {
            let start_pos = batch.len();
            let formatted = formatter.format("", timestamp);
            batch.push_str(&formatted);
            let line_len = (batch.len() - start_pos) as u64;

            if is_filtering && filter_matcher(&batch[start_pos..]) {
                filtered.push(LineRange {
                    start: *current_relative_offset,
                    end: *current_relative_offset + line_len,
                });
            }
            *current_relative_offset = *current_relative_offset + line_len;
            offsets.push(*current_relative_offset);
            return;
        }

        while start < line.len() {
            let mut end = (start + max_len).min(line.len());
            while !line.is_char_boundary(end) {
                end -= 1;
            }
            let sub_line = &line[start..end];

            let start_pos = batch.len();
            let formatted = formatter.format(sub_line, timestamp);
            batch.push_str(&formatted);
            let line_len = (batch.len() - start_pos) as u64;

            if is_filtering && filter_matcher(&batch[start_pos..]) {
                filtered.push(LineRange {
                    start: *current_relative_offset,
                    end: *current_relative_offset + line_len,
                });
            }

            *current_relative_offset = *current_relative_offset + line_len;
            offsets.push(*current_relative_offset);
            start = end;
        }
    }

    pub fn clear(&mut self) {
        self.leftover_buffer.clear();
        // Reset parser state
        self.parser = Parser::new(1, MAX_LINE_BYTES as u16, 0);
    }

    /// Helper to find the next line ending OR buffer full point.
    /// Returns Some((content_end_index, next_start_index)) if found.
    /// - content_end_index: Index exclusive of the newline char(s).
    /// - next_start_index: Index to resume searching for the next line.
    fn find_next_line_ending_or_full(
        chunk: &[u8],
        start: usize,
        remaining_space: usize,
    ) -> Option<(usize, usize)> {
        let len = chunk.len();
        // Check only up to 'remaining_space' or end of chunk, whichever is smaller
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
                        return Some((i, i + 1)); // CR followed by something else
                    }
                } else {
                    // CR at the very end of chunk.
                    // We don't know if next char is LF. Return None to buffer it.
                    return None;
                }
            }
            i += 1;
        }

        // We reached search_limit without finding a newline.
        // If search_limit was determined by remaining_space (i.e., buffer full),
        // we must return a split point here.
        if search_limit == start + remaining_space {
            // Buffer is full. Logic break at search_limit.
            // Ensure we don't split in the middle of a UTF-8 multi-byte character.
            let mut cut_point = search_limit;

            // Check if the cut point lands inside a UTF-8 sequence.
            // We check only the last few bytes (max 3 bytes for 4-byte UTF-8 char) to see if they form a valid sequence end.
            match std::str::from_utf8(&chunk[start..cut_point]) {
                Ok(_) => {} // Valid UTF-8 boundary, safe to cut here.
                Err(e) => {
                    // Valid up to a point, but the last bytes form an incomplete sequence.
                    // We backtrack to exclude the incomplete character from this batch.
                    let valid_len = e.valid_up_to();

                    // Only adjust if the incomplete sequence is at the very end (within 3 bytes).
                    // If it's garbage in the middle, valid_up_to might be far back, which we ignore here.
                    if start + valid_len >= cut_point.saturating_sub(3) {
                        cut_point = start + valid_len;
                    }
                }
            }

            // content_end = cut_point, next_start = cut_point (no delimiter to skip)
            return Some((cut_point, cut_point));
        }

        // Otherwise, buffer is not full yet and no newline found.
        None
    }
}

impl Default for StreamingLineProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worker::formatter::LogFormatterStrategy;

    struct MockFormatter;
    impl LogFormatterStrategy for MockFormatter {
        fn format(&self, text: &str, _timestamp: &str) -> String {
            format!("{}\n", text)
        }
        fn format_chunk(&self, _chunk: &[u8]) -> String {
            String::new()
        }
        fn max_line_length(&self) -> usize {
            1000
        }
    }

    #[test]
    fn test_process_vt100_long_line_wrapping() {
        let mut processor = StreamingLineProcessor::new();
        let formatter = MockFormatter;

        // Use dynamic MAX_LINE_BYTES
        let max_len = MAX_LINE_BYTES;
        let overflow = 44;
        let total_len = max_len + overflow;

        // Feed data larger than buffer
        let data = "a".repeat(total_len);
        let (batch, _, _, active_line) =
            processor.process_vt100(data.as_bytes(), &formatter, "", false, |_| true);

        // Expected behavior:
        // 1. First 'max_len' bytes fill the buffer -> extracted as one line.
        let lines: Vec<&str> = batch.lines().collect();
        assert_eq!(lines.len(), 1, "Should extract exactly one full line");
        assert_eq!(
            lines[0].len(),
            max_len,
            "Extracted line should be MAX_LINE_BYTES long"
        );

        // 2. Remaining bytes are in active_line (since no newline at end)
        assert!(active_line.is_some());
        assert_eq!(
            active_line.unwrap().len(),
            overflow,
            "Remaining bytes should be in active line"
        );
    }

    #[test]
    fn test_process_vt100_long_line_with_newline() {
        let mut processor = StreamingLineProcessor::new();
        let formatter = MockFormatter;

        let max_len = MAX_LINE_BYTES;
        let overflow = 4;

        // Data: [max_len + overflow] 'a' ... '\n' ... [39] 'b'
        // Total 'a' length = max_len + overflow.
        let mut data = "a".repeat(max_len + overflow);
        data.push('\n');
        data.push_str(&"b".repeat(39));

        let (batch, _, _, _) =
            processor.process_vt100(data.as_bytes(), &formatter, "", false, |_| true);

        let lines: Vec<&str> = batch.lines().collect();
        // We expect:
        // 1. Full line of 'a' (max_len)
        // 2. Overflow line of 'a' (overflow len)
        // 3. Line of 'b' - part has no newline, so it remains in active line
        // MockFormatter adds \n for every format() call in extract_and_clear_line.
        // - First split at max_len -> extract -> adds \n
        // - Second split at newline -> extract -> adds \n

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].len(), max_len);
        assert_eq!(lines[1].len(), overflow);
    }

    #[test]
    fn test_process_vt100_prefilled_buffer() {
        let mut processor = StreamingLineProcessor::new();
        let formatter = MockFormatter;
        let max_len = MAX_LINE_BYTES;

        // 1. Prefill buffer with (max_len - 10) bytes
        let initial_fill = max_len - 10;
        let data1 = "A".repeat(initial_fill);

        // First processing: should NOT extract anything yet, just fills buffer
        let (batch1, _, _, active1) =
            processor.process_vt100(data1.as_bytes(), &formatter, "", false, |_| true);

        assert!(batch1.is_empty(), "Should not extract line yet");
        assert_eq!(
            active1.unwrap().len(),
            initial_fill,
            "Active line should contain prefilled data"
        );

        // 2. Feed 20 bytes.
        // Expected:
        // - First 10 bytes fill the remaining space -> Extract 1 full line (max_len)
        // - Remaining 10 bytes start a new line
        let data2 = "B".repeat(20);
        let (batch2, _, _, active2) =
            processor.process_vt100(data2.as_bytes(), &formatter, "", false, |_| true);

        let lines: Vec<&str> = batch2.lines().collect();
        assert_eq!(
            lines.len(),
            1,
            "Should extract exactly one full line after filling remaining space"
        );

        // The extracted line should be: [initial_fill 'A'] + [10 'B']
        let expected_line = format!("{}{}", "A".repeat(initial_fill), "B".repeat(10));
        assert_eq!(lines[0], expected_line);

        // The active line should contain the remaining 10 'B's
        assert!(active2.is_some());
        assert_eq!(active2.unwrap(), "B".repeat(10));
    }

    #[test]
    fn test_process_vt100_empty_chunk() {
        let mut processor = StreamingLineProcessor::new();
        let formatter = MockFormatter;

        let (batch, _, _, _) = processor.process_vt100(b"", &formatter, "", false, |_| true);
        assert!(batch.is_empty());
    }

    #[test]
    fn test_process_vt100_utf8_boundary_handling() {
        let mut processor = StreamingLineProcessor::new();
        let formatter = MockFormatter;
        let max_len = MAX_LINE_BYTES;

        // 1. Fill buffer slightly less than max
        let prefix_len = max_len - 1;
        let prefix = "A".repeat(prefix_len);
        processor.process_vt100(prefix.as_bytes(), &formatter, "", false, |_| true);

        // 2. Next chunk: a 3-byte Hangul char "가" (0xE3, 0x80, 0x80)
        let hangul = "가"; // 3 bytes
        let (batch, _, _, active_line) =
            processor.process_vt100(hangul.as_bytes(), &formatter, "", false, |_| true);

        let lines: Vec<&str> = batch.lines().collect();

        // The split point logic should backtrack from boundary (0+1) to 0.
        // So first line is the prefix "A"s flushed due to buffer full avoidance for "가".
        // Wait, if we return Some((0, 0)), process_vt100 extracts buffer (prefix) and clears.
        // Then loops for remaining chunk ("가").

        assert_eq!(lines.len(), 1, "Should flush the buffer");
        assert_eq!(lines[0].len(), prefix_len);
        assert_eq!(lines[0], prefix);

        // The active line should contain "가" now.
        assert!(active_line.is_some());
        assert_eq!(active_line.unwrap(), "가");
    }
    #[test]
    fn test_process_vt100_stress_mixed_content() {
        let mut processor = StreamingLineProcessor::new();
        let formatter = MockFormatter;

        // 1. Massive Chunk Test (No Newline)
        // Simulate a huge burst of data without newlines (e.g. binary data or glitch)
        // 100KB of 'A's. MAX_LINE_BYTES is 256.
        // It should split into many 256-byte lines without panic.
        let huge_size = 100 * 1024;
        let huge_data = "A".repeat(huge_size);

        let (batch, _, _, _) =
            processor.process_vt100(huge_data.as_bytes(), &formatter, "", false, |_| true);

        // We expect (100*1024 / 256) lines = 400 lines exactly?
        // Let's check line count and length.
        let lines: Vec<&str> = batch.lines().collect();
        assert!(
            lines.len() >= 400,
            "Should split huge line into many fragments"
        );
        for line in &lines {
            // MockFormatter adds \n, but lines() removes it.
            // So content length should be 256 (except maybe last one if math is weird, but here it divides evenly).
            assert_eq!(
                line.len(),
                256,
                "Split line should be exactly MAX_LINE_BYTES"
            );
        }

        // 2. Complex Mixed Content Test
        // - Colors (ANSI)
        // - Multi-byte UTF-8 (Korean, Emoji)
        // - Newlines (\n, \r\n)
        let multi_byte = "안녕하세요 🚀";
        let colored = "\x1b[31mjunk\x1b[0m";
        let mixed_data = format!("Start\n{}{}\nEnd", multi_byte, colored);

        let (batch2, _, _, _) =
            processor.process_vt100(mixed_data.as_bytes(), &formatter, "", false, |_| true);

        let lines2: Vec<&str> = batch2.lines().collect();
        // "Start"
        assert_eq!(lines2[0], "Start");

        // "안녕하세요 🚀junk"
        // VT100 parser regenerates ANSI codes based on cell attributes.
        // It outputs "\u{1b}[31mjunk" (Red color start + text).
        // It seems to omit the reset code (\u{1b}[0m) at the end of the line in this context.
        let expected_mixed = format!("{}\u{1b}[31mjunk", multi_byte);
        assert_eq!(lines2[1], expected_mixed);

        // "End" is active line (no newline after).
    }
    #[test]
    fn test_hex_formatter_newline_handling() {
        use crate::worker::formatter::HexFormatter;
        use crate::worker::formatter::LogFormatterStrategy;

        let formatter = HexFormatter { max_bytes: 16 };

        // Input with newlines and carriage returns
        let input = b"A\nB\rC"; // 0x41, 0x0A, 0x42, 0x0D, 0x43

        let formatted = formatter.format_chunk(input);

        // Expected: "41 0A 42 0D 43 "
        assert_eq!(formatted, "41 0A 42 0D 43 ");

        assert_eq!(formatter.max_line_length(), 48);
    }

    struct MockHexFormatter;
    impl crate::worker::formatter::LogFormatterStrategy for MockHexFormatter {
        fn format(&self, text: &str, _timestamp: &str) -> String {
            format!("{}\n", text)
        }
        fn format_chunk(&self, _chunk: &[u8]) -> String {
            String::new()
        }
        fn max_line_length(&self) -> usize {
            16 * 3 // 48
        }
    }

    #[test]
    fn test_hex_mode_fixed_width_splitting() {
        let mut processor = StreamingLineProcessor::new();
        let formatter = MockHexFormatter;

        // Input: "00 11 22 33 44 55 66 77 88 99 AA BB CC DD EE FF " (48 chars)
        let line1 = "00 11 22 33 44 55 66 77 88 99 AA BB CC DD EE FF ";
        // Input 2: "10 11 12 13 " (12 chars)
        let line2_part = "10 11 12 13 ";

        let full_text = format!("{}{}", line1, line2_part); // 60 chars

        // process_hex_lines takes &str, splits by 48 chars.
        // It now returns active_line (leftover buffer).
        let (batch, _, _, active) =
            processor.process_hex_lines(&full_text, &formatter, "", false, |_| true);

        let lines: Vec<&str> = batch.lines().collect();
        assert_eq!(lines.len(), 1);
        // lines[0] contains "00 .. FF ", which matches line1 exactly.
        assert_eq!(lines[0], line1);

        // Check buffer for remainder. It should be returned as active_line too.
        assert_eq!(processor.leftover_buffer, line2_part);
        assert_eq!(active, Some(line2_part.to_string()));

        // Send next chunk to complete line 2
        // We need 36 chars more. "20 .. 2B "
        let line3_part = "20 21 22 23 24 25 26 27 28 29 2A 2B ";
        let (batch2, _, _, active2) =
            processor.process_hex_lines(line3_part, &formatter, "", false, |_| true);

        let lines2: Vec<&str> = batch2.lines().collect();
        assert_eq!(lines2.len(), 1);

        let expected_line2 = format!("{}{}", line2_part, line3_part);
        // lines2[0] contains "10 .. 2B ", which matches expected_line2 exactly.
        assert_eq!(lines2[0], expected_line2);

        assert!(processor.leftover_buffer.is_empty());
    }
}
