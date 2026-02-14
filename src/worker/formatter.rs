use chrono::Timelike;
use std::fmt::Write;

pub trait LogFormatterStrategy {
    fn format(&self, text: &str, timestamp: &str) -> String;
    fn format_chunk(&self, chunk: &[u8]) -> String;
    fn max_line_length(&self) -> usize;
}

pub struct DefaultFormatter {
    pub max_bytes: usize,
}

impl LogFormatterStrategy for DefaultFormatter {
    fn format(&self, text: &str, timestamp: &str) -> String {
        if timestamp.is_empty() {
            format!("{}\n", text)
        } else {
            format!("{} {}\n", timestamp, text)
        }
    }

    fn format_chunk(&self, _chunk: &[u8]) -> String {
        String::new()
    }

    fn max_line_length(&self) -> usize {
        self.max_bytes
    }
}

pub struct HexFormatter {
    pub max_bytes: usize,
}

impl LogFormatterStrategy for HexFormatter {
    fn format(&self, text: &str, timestamp: &str) -> String {
        if timestamp.is_empty() {
            format!("{}\n", text)
        } else if text.is_empty() {
            format!("{}\n", timestamp)
        } else {
            format!("{} {}\n", timestamp, text)
        }
    }

    fn format_chunk(&self, chunk: &[u8]) -> String {
        let mut acc = String::with_capacity(chunk.len() * 3);
        for &b in chunk {
            // Hex view should display ALL bytes as hex codes, including control characters.
            let _ = write!(acc, "{:02X} ", b);
        }
        acc
    }

    fn max_line_length(&self) -> usize {
        // Standard Hex View: 16 bytes per line
        // Each byte is 2 hex chars + 1 space = 3 chars
        16 * 3
    }
}

pub struct LogFormatter;

impl LogFormatter {
    pub fn new() -> Self {
        Self
    }

    pub fn get_timestamp(&self) -> String {
        let now = chrono::Utc::now();
        format!(
            "[{:02}:{:02}:{:02}.{:03}]",
            now.hour(),
            now.minute(),
            now.second(),
            now.timestamp_subsec_millis()
        )
    }

    pub fn create_strategy(&self, is_hex: bool, max_bytes: usize) -> Box<dyn LogFormatterStrategy> {
        if is_hex {
            Box::new(HexFormatter { max_bytes })
        } else {
            Box::new(DefaultFormatter { max_bytes })
        }
    }
}
