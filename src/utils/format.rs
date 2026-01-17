pub fn format_hex(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<String>>()
        .join(" ")
}

pub fn parse_hex_string(input: &str) -> Result<Vec<u8>, String> {
    // Remove allowed separators (space, colon, dash) and 0x prefix
    let clean = input
        .replace(" ", "")
        .replace(":", "")
        .replace("-", "")
        .replace("0x", "");

    // Check length (must be even for bytes) -> actually relaxed?
    // User might type "A" -> "0A"?
    // If strict:
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_hex() {
        assert_eq!(format_hex(&[0x0A, 0xFF, 0x00]), "0A FF 00");
        assert_eq!(format_hex(&[]), "");
    }
}
