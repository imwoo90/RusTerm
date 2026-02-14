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
        // Standard Hex View: max_bytes per line (usually 16)
        // Each byte is 2 hex chars + 1 space = 3 chars
        self.max_bytes * 3
    }
}

pub struct LogFormatter;

impl LogFormatter {
    pub fn new() -> Self {
        Self
    }

    pub fn get_timestamp(&self) -> String {
        let now = js_sys::Date::new_0();
        format!(
            "[{:02}:{:02}:{:02}.{:03}]",
            now.get_hours(),
            now.get_minutes(),
            now.get_seconds(),
            now.get_milliseconds()
        )
    }

    pub fn create_strategy(&self, is_hex: bool) -> Box<dyn LogFormatterStrategy> {
        if is_hex {
            // Hex Mode uses fixed HEX_VIEW_BYTES from config
            Box::new(HexFormatter {
                max_bytes: crate::config::HEX_VIEW_BYTES,
            })
        } else {
            Box::new(DefaultFormatter {
                max_bytes: crate::config::MAX_LINE_BYTES,
            })
        }
    }
}
