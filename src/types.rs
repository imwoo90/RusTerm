use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Highlight {
    pub id: usize,
    pub text: String,
    pub color: &'static str,
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum LineEnding {
    #[default]
    None,
    NL,
    CR,
    NLCR,
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum Parity {
    #[default]
    None,
    Even,
    Odd,
}

impl ToString for Parity {
    fn to_string(&self) -> String {
        match self {
            Parity::None => "none".to_string(),
            Parity::Even => "even".to_string(),
            Parity::Odd => "odd".to_string(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum FlowControl {
    #[default]
    None,
    Hardware,
}

impl ToString for FlowControl {
    fn to_string(&self) -> String {
        match self {
            FlowControl::None => "none".to_string(),
            FlowControl::Hardware => "hardware".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum WorkerMsg {
    NewSession,
    AppendChunk {
        chunk: Vec<u8>,
        is_hex: bool,
    },
    AppendLog(String),
    RequestWindow {
        start_line: usize,
        count: usize,
    },
    LogWindow {
        start_line: usize,
        lines: Vec<(usize, String)>,
    },
    TotalLines(usize),
    Clear,
    SetLineEnding(String),
    SearchLogs {
        query: String,
        match_case: bool,
        use_regex: bool,
        invert: bool,
    },
    ExportLogs {
        include_timestamp: bool,
    },
    Error(String),
}
