//! Unit tests for streaming line processor, VT100 parser, and hex chunking.

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::config::MAX_LINE_BYTES;
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

        let max_len = MAX_LINE_BYTES;
        let overflow = 44;
        let total_len = max_len + overflow;

        let data = "a".repeat(total_len);
        let (batch, _, _, active_line) =
            processor.process_vt100(data.as_bytes(), &formatter, "", false, |_| true);

        let lines: Vec<&str> = batch.lines().collect();
        assert_eq!(lines.len(), 1, "Should extract exactly one full line");
        assert_eq!(lines[0].len(), max_len);
        assert!(active_line.is_some());
        assert_eq!(active_line.unwrap().len(), overflow);
    }

    #[test]
    fn test_process_vt100_long_line_with_newline() {
        let mut processor = StreamingLineProcessor::new();
        let formatter = MockFormatter;

        let max_len = MAX_LINE_BYTES;
        let overflow = 4;

        let mut data = "a".repeat(max_len + overflow);
        data.push('\n');
        data.push_str(&"b".repeat(39));

        let (batch, _, _, _) =
            processor.process_vt100(data.as_bytes(), &formatter, "", false, |_| true);

        let lines: Vec<&str> = batch.lines().collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].len(), max_len);
        assert_eq!(lines[1].len(), overflow);
    }

    #[test]
    fn test_process_vt100_prefilled_buffer() {
        let mut processor = StreamingLineProcessor::new();
        let formatter = MockFormatter;
        let max_len = MAX_LINE_BYTES;

        let initial_fill = max_len - 10;
        let data1 = "A".repeat(initial_fill);

        let (batch1, _, _, active1) =
            processor.process_vt100(data1.as_bytes(), &formatter, "", false, |_| true);

        assert!(batch1.is_empty(), "Should not extract line yet");
        assert_eq!(active1.unwrap().len(), initial_fill);

        let data2 = "B".repeat(20);
        let (batch2, _, _, active2) =
            processor.process_vt100(data2.as_bytes(), &formatter, "", false, |_| true);

        let lines: Vec<&str> = batch2.lines().collect();
        assert_eq!(lines.len(), 1);
        let expected_line = format!("{}{}", "A".repeat(initial_fill), "B".repeat(10));
        assert_eq!(lines[0], expected_line);
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

        let prefix_len = max_len - 1;
        let prefix = "A".repeat(prefix_len);
        processor.process_vt100(prefix.as_bytes(), &formatter, "", false, |_| true);

        let hangul = "가";
        let (batch, _, _, active_line) =
            processor.process_vt100(hangul.as_bytes(), &formatter, "", false, |_| true);

        let lines: Vec<&str> = batch.lines().collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].len(), prefix_len);
        assert_eq!(lines[0], prefix);
        assert_eq!(active_line.unwrap(), "가");
    }

    #[test]
    fn test_process_vt100_massive_burst() {
        let mut processor = StreamingLineProcessor::new();
        let formatter = MockFormatter;

        let huge_size = 100 * 1024;
        let huge_data = "A".repeat(huge_size);

        let (batch, _, _, _) =
            processor.process_vt100(huge_data.as_bytes(), &formatter, "", false, |_| true);

        let lines: Vec<&str> = batch.lines().collect();
        assert!(lines.len() >= 400, "Should split huge line into fragments");
        for line in &lines {
            assert_eq!(line.len(), 256, "Split line should be MAX_LINE_BYTES");
        }
    }

    #[test]
    fn test_process_vt100_mixed_multibyte_ansi() {
        let mut processor = StreamingLineProcessor::new();
        let formatter = MockFormatter;

        let multi_byte = "안녕하세요 🚀";
        let colored = "\x1b[31mjunk\x1b[0m";
        let mixed_data = format!("Start\n{}{}\nEnd", multi_byte, colored);

        let (batch, _, _, _) =
            processor.process_vt100(mixed_data.as_bytes(), &formatter, "", false, |_| true);

        let lines: Vec<&str> = batch.lines().collect();
        assert_eq!(lines[0], "Start");
        let expected_mixed = format!("{}\u{1b}[31mjunk", multi_byte);
        assert_eq!(lines[1], expected_mixed);
    }

    #[test]
    fn test_hex_formatter_newline_handling() {
        use crate::worker::formatter::HexFormatter;
        use crate::worker::formatter::LogFormatterStrategy;

        let formatter = HexFormatter { max_bytes: 16 };
        let input = b"A\nB\rC";
        let formatted = formatter.format_chunk(input);
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
            48
        }
    }

    #[test]
    fn test_hex_mode_fixed_width_splitting() {
        let mut processor = StreamingLineProcessor::new();
        let formatter = MockHexFormatter;

        let line1 = "00 11 22 33 44 55 66 77 88 99 AA BB CC DD EE FF ";
        let line2_part = "10 11 12 13 ";
        let full_text = format!("{}{}", line1, line2_part);

        let (batch, _, _, active) =
            processor.process_hex_lines(&full_text, &formatter, "", false, |_| true);

        let lines: Vec<&str> = batch.lines().collect();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], line1);
        assert_eq!(processor.leftover_buffer, line2_part);
        assert_eq!(active, Some(line2_part.to_string()));

        let line3_part = "20 21 22 23 24 25 26 27 28 29 2A 2B ";
        let (batch2, _, _, active2) =
            processor.process_hex_lines(line3_part, &formatter, "", false, |_| true);
        assert!(active2.is_none());

        let lines2: Vec<&str> = batch2.lines().collect();
        assert_eq!(lines2.len(), 1);
        let expected_line2 = format!("{}{}", line2_part, line3_part);
        assert_eq!(lines2[0], expected_line2);
        assert!(processor.leftover_buffer.is_empty());
    }
}
