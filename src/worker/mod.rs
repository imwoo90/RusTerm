//! Web Worker backend subsystem catalog (mod.rs = index.md).
//!
//! Implements a dedicated background Web Worker handling high-throughput log ingestion,
//! OPFS persistence, indexing, regex searching, and formatted batch delivery.

#![allow(dead_code)]

pub mod chunk_handler;
pub mod commands;
pub mod dispatcher;
pub mod error;
pub mod export;
pub mod formatter;
pub mod lifecycle;
pub mod processor;
pub mod repository;
pub mod search;
pub mod state;
pub mod types;

// Re-export public functions
pub use lifecycle::get_app_script_path;
