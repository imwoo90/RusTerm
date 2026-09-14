//! Sub-line formatting and indexing logic for streaming chunks.
//!
//! Handles line length clamping, UTF-8 char boundary adjustments,
//! and writes formatted outputs alongside byte offsets and filter matches.

use crate::worker::formatter::LogFormatterStrategy;
use crate::worker::repository::index::{ByteOffset, LineRange};

/// Splits and formats a single logical line, appending to batch and offset trackers.
#[allow(clippy::too_many_arguments)]
pub fn format_and_append_line(
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

    if line.is_empty() {
        append_single_subline(
            "",
            formatter,
            timestamp,
            batch,
            offsets,
            filtered,
            current_relative_offset,
            is_filtering,
            filter_matcher,
        );
        return;
    }

    while start < line.len() {
        let mut end = (start + max_len).min(line.len());
        while !line.is_char_boundary(end) {
            end -= 1;
        }
        let sub_line = &line[start..end];
        append_single_subline(
            sub_line,
            formatter,
            timestamp,
            batch,
            offsets,
            filtered,
            current_relative_offset,
            is_filtering,
            filter_matcher,
        );
        start = end;
    }
}

#[allow(clippy::too_many_arguments)]
fn append_single_subline(
    sub_line: &str,
    formatter: &dyn LogFormatterStrategy,
    timestamp: &str,
    batch: &mut String,
    offsets: &mut Vec<ByteOffset>,
    filtered: &mut Vec<LineRange>,
    current_relative_offset: &mut ByteOffset,
    is_filtering: bool,
    filter_matcher: &impl Fn(&str) -> bool,
) {
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
}
