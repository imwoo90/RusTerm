use crate::worker::formatter::LogFormatterStrategy;
use crate::worker::index::{ByteOffset, LineRange};
use std::borrow::Cow;

/// Handles chunk processing and line splitting logic
pub struct ChunkHandler {
    pub leftover_chunk: String,
}

impl ChunkHandler {
    pub fn new() -> Self {
        Self {
            leftover_chunk: String::new(),
        }
    }

    /// Prepares a batch of formatted lines from a chunk
    pub fn prepare_batch_with_formatter(
        &mut self,
        chunk: &str,
        formatter: &dyn LogFormatterStrategy,
        timestamp: &str,
        is_filtering: bool,
        filter_matcher: impl Fn(&str) -> bool,
    ) -> (String, Vec<ByteOffset>, Vec<LineRange>) {
        let max_len = formatter.max_line_length();

        // 1. If leftover is already too long, force a split before even adding new chunk
        if !self.leftover_chunk.is_empty() && self.leftover_chunk.len() >= max_len {
            self.leftover_chunk.push('\n');
        }

        let full_text = if self.leftover_chunk.is_empty() {
            Cow::Borrowed(chunk)
        } else {
            Cow::Owned(format!("{}{}", self.leftover_chunk, chunk))
        };

        let mut raw_lines: Vec<&str> = self.split_by_line_ending(&full_text, formatter);

        // The last part is the new leftover
        self.leftover_chunk = raw_lines.pop().unwrap_or("").to_string();

        let mut batch = String::with_capacity(full_text.len() * 2);
        let mut offsets = Vec::with_capacity(raw_lines.len());
        let mut filtered = Vec::new();
        let mut relative_offset = ByteOffset(0);

        for line in raw_lines {
            let cleaned = formatter.clean_line_ending(line);
            self.process_single_line(
                cleaned,
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

        (batch, offsets, filtered)
    }

    fn split_by_line_ending<'a>(
        &self,
        text: &'a str,
        _formatter: &dyn LogFormatterStrategy,
    ) -> Vec<&'a str> {
        // We need to get the line ending mode from the formatter
        // For now, we'll use a heuristic based on the formatter type
        // This is a temporary solution - ideally formatter should expose line_ending

        // Default to NL split
        text.split('\n').collect()
    }

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
            let end = (start + max_len).min(line.len());
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
        self.leftover_chunk.clear();
    }
}

impl Default for ChunkHandler {
    fn default() -> Self {
        Self::new()
    }
}
