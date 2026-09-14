//! # Worker Subsystem Catalog (index.md)
//!
//! ## Overview
//! Dedicated background Web Worker thread handling high-throughput serial log ingestion,
//! OPFS binary persistence, indexed search offsets, and formatted batch delivery to UI.
//!
//! ## Submodules
//! - [`chunk_handler`]: Incoming serial chunk buffering, VT100 ANSI parsing, and UTF-8 line splitting.
//! - [`processor`]: Core ingestion pipeline loop (`LogProcessor`) coordinating chunk handlers and storage.
//! - [`repository`]: High-throughput log storage engine backed by OPFS and indexed offset lookup.
//! - [`commands`]: Typed command pattern implementations for worker request/response messaging.
//! - [`dispatcher`]: Top-level worker message routing from web worker `onmessage` events.
//! - [`search`]: Regex-powered log searching and active filter execution.
//! - [`formatter`]: Log display formatting strategies (ASCII, ANSI VT100, Hex dump).
//! - [`export`]: Log file download and export generation (text/raw).
//! - [`lifecycle`]: Web Worker initialization, script path resolution, and error handlers.
//! - [`state`]: Thread-local worker mutable state containers.
//! - [`types`]: Public IPC message types and log chunk envelopes.
//! - [`error`]: Worker domain error types and Result aliases.
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
pub mod types;

// Re-export public functions
pub use lifecycle::get_app_script_path;
