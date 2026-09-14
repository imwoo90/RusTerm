//! Delimiter scanning and UTF-8 safe chunk slicing for stream processors.
//!
//! Scans byte slices for newline delimiters (`\n`, `\r`, `\r\n`) within buffer limits
//! and calculates safe split boundaries that preserve multi-byte UTF-8 code point integrity.

/// Scans for next newline delimiter or reports buffer full boundary.
/// Returns `Some((content_end_index, next_start_index))` if found.
pub fn find_next_line_ending_or_full(
    chunk: &[u8],
    start: usize,
    remaining_space: usize,
) -> Option<(usize, usize)> {
    let len = chunk.len();
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
                    return Some((i, i + 1)); // CR
                }
            } else {
                // Trailing CR: buffer to next chunk
                return None;
            }
        }
        i += 1;
    }

    if search_limit == start + remaining_space {
        let cut = find_safe_utf8_cut(chunk, start, search_limit);
        return Some((cut, cut));
    }

    None
}

/// Backtracks up to 3 bytes if cut lands inside an incomplete UTF-8 multi-byte sequence.
fn find_safe_utf8_cut(chunk: &[u8], start: usize, search_limit: usize) -> usize {
    let mut cut_point = search_limit;
    match std::str::from_utf8(&chunk[start..cut_point]) {
        Ok(_) => cut_point,
        Err(e) => {
            let valid_len = e.valid_up_to();
            if start + valid_len >= cut_point.saturating_sub(3) {
                cut_point = start + valid_len;
            }
            cut_point
        }
    }
}
