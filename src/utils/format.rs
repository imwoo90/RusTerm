// function format_hex removed

pub fn parse_hex_string(input: &str) -> Result<Vec<u8>, String> {
    // ... same as before
    // Remove allowed separators (space, colon, dash) and 0x prefix
    let clean = input
        .replace(" ", "")
        .replace(":", "")
        .replace("-", "")
        .replace("0x", "");

    if clean.len() % 2 != 0 {
        return Err("Hex string must have an even number of characters".to_string());
    }

    (0..clean.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&clean[i..i + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
        .map_err(|_| "Invalid hex character".to_string())
}

pub fn format_hex_input(input: &str) -> String {
    let clean: String = input
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .map(|c| c.to_ascii_uppercase())
        .collect();

    clean
        .as_bytes()
        .chunks(2)
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
        .collect::<Vec<&str>>()
        .join(" ")
}

/// Helper to send raw byte chunk to worker
pub fn send_chunk_to_worker(worker: &web_sys::Worker, data: &[u8], is_hex: bool) {
    if let Ok(msg) =
        serde_json::to_string(&crate::components::console::types::WorkerMsg::AppendChunk {
            chunk: data.to_vec(),
            is_hex,
        })
    {
        let _ = worker.post_message(&msg.into());
    }
}

/// Helper to send general control messages to worker
pub fn send_worker_msg(
    worker: &web_sys::Worker,
    msg: crate::components::console::types::WorkerMsg,
) {
    if let Ok(msg_str) = serde_json::to_string(&msg) {
        let _ = worker.post_message(&msg_str.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // test_format_hex removed

    #[test]
    fn test_parse_hex_string() {
        // Valid cases
        assert_eq!(parse_hex_string("AA BB CC"), Ok(vec![0xAA, 0xBB, 0xCC]));
        assert_eq!(parse_hex_string("aa:bb-cc"), Ok(vec![0xAA, 0xBB, 0xCC]));
        assert_eq!(parse_hex_string("0xDE 0xAD"), Ok(vec![0xDE, 0xAD]));
        assert_eq!(
            parse_hex_string("deadbeef"),
            Ok(vec![0xDE, 0xAD, 0xBE, 0xEF])
        );
        assert_eq!(parse_hex_string(""), Ok(vec![]));

        // Invalid cases
        assert!(parse_hex_string("ABC").is_err()); // Odd length
        assert!(parse_hex_string("G H I").is_err()); // Invalid chars
        assert!(parse_hex_string("0").is_err()); // Odd length
        assert!(parse_hex_string("0xGG").is_err()); // Invalid chars
    }

    #[test]
    fn test_format_hex_input() {
        // Formatting logic: formatting happens on input, so it should space out every 2 chars
        assert_eq!(format_hex_input("a"), "A");
        assert_eq!(format_hex_input("ab"), "AB");
        assert_eq!(format_hex_input("abc"), "AB C");
        assert_eq!(format_hex_input("abcd"), "AB CD");
        assert_eq!(format_hex_input("abcde"), "AB CD E");

        // Handles mixed case and separators gracefully (by removing them then formatting)
        assert_eq!(format_hex_input("a b c d"), "AB CD");
        // Let's verify 'x' behavior. 'x' is not a hexdigit.
        // So "0xab" -> '0', 'a', 'b' -> "0A B" or "0AB"?
        // 0 (hex), x (skip), a (hex), b (hex).
        // chars: 0, a, b. -> "0A B".
        assert_eq!(format_hex_input("0xab"), "0A B");

        assert_eq!(format_hex_input("hello world"), "ED"); // h(skip), e(E), l(skip)... d(D).
                                                           // e, d. -> ED
    }
}
