//! # Core Domain Types & Models
//!
//! ## Overview
//! Contains shared enumerations and helper models used across both UI presentation layers and background worker communication channels.
//!
//! ## Search Tags
//! #types, #models, #domain, #enums, #serial-types

use serde::{Deserialize, Serialize};
use std::fmt;

/// Keyword highlight rule with assigned display color.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Highlight {
    /// Unique identifier for the highlight entry.
    pub id: usize,
    /// Target substring or pattern to highlight.
    pub text: String,
    /// Static CSS color class or hex string.
    pub color: &'static str,
}

/// Line termination sequence appended to outgoing serial transmissions.
#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum LineEnding {
    /// Do not append any line terminator.
    #[default]
    None,
    /// Append Line Feed (`\n`).
    NL,
    /// Append Carriage Return (`\r`).
    CR,
    /// Append Line Feed followed by Carriage Return (`\n\r`).
    NLCR,
}

/// Active workspace presentation mode.
#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum ViewMode {
    /// Structured log monitoring view with filtering and virtual scrolling.
    #[default]
    Monitoring,
    /// Interactive VT100 terminal emulation view.
    Terminal,
}

/// Serial port parity checking mode.
#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum Parity {
    /// No parity bit.
    #[default]
    None,
    /// Even parity bit.
    Even,
    /// Odd parity bit.
    Odd,
}

impl fmt::Display for Parity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Parity::None => write!(f, "none"),
            Parity::Even => write!(f, "even"),
            Parity::Odd => write!(f, "odd"),
        }
    }
}

/// Serial port hardware handshake flow control setting.
#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum FlowControl {
    /// Flow control disabled.
    #[default]
    None,
    /// RTS/CTS hardware flow control enabled.
    Hardware,
}

impl fmt::Display for FlowControl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlowControl::None => write!(f, "none"),
            FlowControl::Hardware => write!(f, "hardware"),
        }
    }
}

/// Asynchronous IPC message envelope exchanged between the UI thread and Web Worker.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum WorkerMsg {
    /// Resets session state for a fresh serial connection.
    NewSession,
    /// Appends raw incoming serial bytes for parsing and persistence.
    AppendChunk {
        /// Raw byte payload received from Web Serial.
        chunk: Vec<u8>,
        /// Whether to format payload as hexadecimal dump.
        is_hex: bool,
    },
    /// Enables or disables log line timestamp prefixing.
    SetTimestampState(bool),

    /// Requests a slice of log lines for virtual scroll rendering.
    RequestWindow {
        /// Starting zero-based line index.
        start_line: usize,
        /// Number of lines to fetch.
        count: usize,
    },
    /// Returns a window of formatted log lines to the UI.
    LogWindow {
        /// Starting line index of the window.
        start_line: usize,
        /// Pairs of `(original_line_number, formatted_line_text)`.
        lines: Vec<(usize, String)>,
    },
    /// Reports total stored or filtered line count.
    TotalLines(usize),
    /// Clears all stored log history and indices.
    Clear,
    /// Applies a search/filter query over the log repository.
    SearchLogs {
        /// Filter pattern string.
        query: String,
        /// Whether matching is case-sensitive.
        match_case: bool,
        /// Whether to interpret `query` as a regular expression.
        use_regex: bool,
        /// Whether to invert matches (exclude matching lines).
        invert: bool,
    },
    /// Exports stored logs to a downloadable file.
    ExportLogs {
        /// Whether exported lines should retain timestamps.
        include_timestamp: bool,
    },
    /// Emits the currently incomplete active line buffer.
    ActiveLine(Option<String>),
    /// Switches worker processing mode between monitoring and terminal.
    SetMode(ViewMode),
    /// Reports a worker error message to the UI.
    Error(String),
}
