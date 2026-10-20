//! # Hex Stream Processor
//!
//! ## Overview
//! Splits streaming hex output into fixed-width lines based on formatter configuration,
//! retaining partial leftover chunks across stream frames.
//!
//! ## Search Tags
//! #hex, #hex-stream, #fixed-width, #leftover-buffer

use crate::worker::chunk_handler::line_builder::LineBatchAccumulator;
use crate::worker::formatter::LogFormatterStrategy;
use crate::worker::repository::index::{ByteOffset, LineRange};
use std::borrow::Cow;

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
