//! # ANSI Color & Style Decoder
//!
//! ## Overview
//! Parses ANSI escape sequences into styled text spans and applies user highlight rules to break lines into formatted visual segments.
//!
//! ## Search Tags
//! #ansi-decoder, #escape-sequences, #styling, #color-parser, #highlights

use crate::state::Highlight;
use regex::Regex;

fn sgr_color(code: &str) -> Option<&'static str> {
    match code {
        "30" | "90" => Some("#9ca3af"),
        "31" | "91" => Some("#ef4444"),
        "32" | "92" => Some("#10b981"),
        "33" | "93" => Some("#f59e0b"),
        "34" | "94" => Some("#3b82f6"),
        "35" | "95" => Some("#d946ef"),
        "36" | "96" => Some("#06b6d4"),
        "37" | "97" => Some("#f3f4f6"),
        _ => None,
    }
}

fn parse_sgr_codes(params: &str, current_color: &mut Option<String>) {
    if params.is_empty() {
        *current_color = None;
    } else {
        for code in params.split(';') {
            if code == "0" {
                *current_color = None;
            } else if let Some(hex) = sgr_color(code) {
                *current_color = Some(hex.to_string());
            }
        }
    }
}

fn parse_ansi_segments(content: &str) -> Vec<(String, Option<String>)> {
    let mut segments = Vec::new();

    thread_local! {
        static ANSI_RE: Regex = Regex::new(r"\x1B\[([0-9;]*)([A-Za-z])").unwrap();
    }

    let mut last_pos = 0;
    let mut current_color: Option<String> = None;

    ANSI_RE.with(|re| {
        for cap in re.captures_iter(content) {
            let m = match cap.get(0) {
                Some(m) => m,
                None => continue,
            };
            let start = m.start();
            let end = m.end();

            if start > last_pos {
                segments.push((content[last_pos..start].to_string(), current_color.clone()));
            }

            if let Some(cmd_match) = cap.get(2) {
                let params = cap.get(1).map_or("", |m| m.as_str());
                match cmd_match.as_str() {
                    "m" => parse_sgr_codes(params, &mut current_color),
                    "C" => {
                        let count = params.parse::<usize>().unwrap_or(1);
                        segments.push((" ".repeat(count), current_color.clone()));
                    }
                    _ => {}
                }
            }

            last_pos = end;
        }
    });

    if last_pos < content.len() {
        segments.push((content[last_pos..].to_string(), current_color));
    } else if segments.is_empty() && !content.is_empty() {
        segments.push((content.to_string(), None));
    }

    segments
}

fn apply_user_highlights(
    mut segments: Vec<(String, Option<String>)>,
    highlights: &[Highlight],
) -> Vec<(String, Option<String>)> {
    for h in highlights {
        if h.text.is_empty() {
            continue;
        }

        let mut next_segments = Vec::new();

        for (seg_text, color) in segments {
            if seg_text.contains(&h.text) {
                if let Some((prefix, suffix)) = seg_text.split_once(&h.text) {
                    if !prefix.is_empty() {
                        next_segments.push((prefix.to_string(), color.clone()));
                    }
                    next_segments.push((h.text.clone(), Some(h.color.to_string())));
                    if !suffix.is_empty() {
                        next_segments.push((suffix.to_string(), color.clone()));
                    }
                } else {
                    next_segments.push((seg_text, color));
                }
            } else {
                next_segments.push((seg_text, color));
            }
        }
        segments = next_segments;
    }
    segments
}

/// Processes log text to parse ANSI colors and apply user highlights
pub fn decode_ansi_text(
    text: &str,
    highlights: &[Highlight],
    show_highlights: bool,
) -> Vec<(String, Option<String>)> {
    let segments = parse_ansi_segments(text);
    if show_highlights {
        apply_user_highlights(segments, highlights)
    } else {
        segments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_parsing() {
        let input = "\x1B[31mRed Text\x1B[0m Normal Text";
        let result = decode_ansi_text(input, &[], false);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "Red Text");
        assert_eq!(result[0].1, Some("#ef4444".to_string()));
        assert_eq!(result[1].0, " Normal Text");
        assert_eq!(result[1].1, None);
    }

    #[test]
    fn test_highlight_overlay() {
        let input = "Error: Something failed";
        let highlights = vec![Highlight {
            id: 1,
            text: "Error".to_string(),
            color: "red",
        }];

        let result = decode_ansi_text(input, &highlights, true);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "Error");
        assert_eq!(result[0].1, Some("red".to_string()));
        assert_eq!(result[1].0, ": Something failed");
        assert_eq!(result[1].1, None);
    }
}
