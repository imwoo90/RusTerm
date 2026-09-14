//! Worker messaging types and inter-thread IPC protocols.
//!
//! Defines message schemas exchanged between the main UI thread and background Web Worker
//! via postMessage serialization.

pub use crate::types::WorkerMsg;
