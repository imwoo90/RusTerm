use serde::{Deserialize, Serialize};

/// Message protocol for communicating with Web Worker
/// Note: Do NOT use #[serde(tag = "...")] or #[serde(rename = "...")]
/// as gloo-worker's default Bincode codec does not support them.
#[derive(Serialize, Deserialize, Debug)]
pub enum WorkerMsg {
    TotalLines(usize),
    LogWindow {
        start_line: usize,
        lines: Vec<String>,
    },
    AppendLog(String),
    RequestWindow {
        start_line: usize,
        count: usize,
    },
    ExportLogs {
        include_timestamp: bool,
    },
    Clear,
    Error(String),
    SearchLogs {
        query: String,
        match_case: bool,
        use_regex: bool,
        invert: bool,
    },
    SetLineEnding(String),
    NewSession,
    AppendChunk {
        chunk: Vec<u8>,
        is_hex: bool,
    },
}

pub const LINE_HEIGHT: f64 = 20.0;
pub const HEADER_OFFSET: f64 = 150.0;
pub const TOP_BUFFER: usize = 10;
pub const BOTTOM_BUFFER_EXTRA: usize = 40;
pub const CONSOLE_TOP_PADDING: f64 = 8.0; // 0.5rem
pub const CONSOLE_BOTTOM_PADDING: f64 = 20.0;
