//! Streaming line processor and chunk coordinator for worker log ingestion.
//!
//! Provides the primary [`StreamingLineProcessor`] which buffers incoming serial chunks,
//! handles terminal escape code parsing via VT100 emulation, splits lines safely along
//! UTF-8 and maximum byte boundaries, and formats logs for display and OPFS storage.

mod hex;
mod line_builder;
mod scanner;
#[cfg(test)]
mod tests;
mod vt100_stream;

use crate::config::MAX_LINE_BYTES;
use crate::worker::formatter::LogFormatterStrategy;
use crate::worker::repository::index::{ByteOffset, LineRange};
use ::vt100::Parser;

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
        vt100_stream::process_vt100_stream(
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
        hex::process_hex_stream(
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
