//! # Worker Subsystem Catalog (index.md)
//!
//! ## Overview
//! Dedicated background Web Worker thread handling high-throughput serial log ingestion,
//! OPFS binary persistence, indexed search offsets, and formatted batch delivery to UI.
//!
//! ## Submodules
//! - [`chunk_handler`] (`chunk_handler/mod.rs`): Incoming serial chunk buffering, VT100 ANSI parsing, and UTF-8 line splitting.
//! - [`processor`] (`processor.rs`): Core ingestion pipeline loop (`LogProcessor`) coordinating chunk handlers and storage.
//! - [`repository`] (`repository/mod.rs`): High-throughput log storage engine backed by OPFS and indexed offset lookup.
//! - [`commands`] (`commands/mod.rs`): Typed command pattern implementations for worker request/response messaging.
//! - [`dispatcher`] (`dispatcher.rs`): Top-level worker message routing from web worker `onmessage` events.
//! - [`search`] (`search.rs`): Regex-powered log searching and active filter execution.
//! - [`formatter`] (`formatter.rs`): Log display formatting strategies (ASCII, ANSI VT100, Hex dump).
//! - [`export`] (`export.rs`): Log file download and export generation (text/raw).
//! - [`lifecycle`] (`lifecycle.rs`): Web Worker initialization, script path resolution, and error handlers.
//! - [`state`] (`state.rs`): Thread-local worker mutable state containers.
//! - [`error`] (`error.rs`): Worker domain error types and Result aliases.
//!
//! ## Search Tags
//! #worker, #log-ingestion, #opfs, #vt100, #ipc, #pipeline

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

// Re-export public functions
pub use lifecycle::get_app_script_path;
