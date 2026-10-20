//! # Line Builder & Formatter
//!
//! ## Overview
//! Handles line length clamping, UTF-8 char boundary adjustments,
//! and writes formatted outputs alongside byte offsets and filter matches.
//!
//! ## Search Tags
//! #line-builder, #line-clamping, #utf8-boundary, #offset-indexing

use crate::worker::formatter::LogFormatterStrategy;
use crate::worker::repository::index::{ByteOffset, LineRange};

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
