//! # Reactive Hooks Catalog (index.md)
//!
//! ## Overview
//! Re-exports domain controllers and async workers managing Web Serial connections, Web Worker messaging bridges, and reactive component lifecycles.
//!
//! ## Submodules
//! - [`serial`]: Web Serial port connection state, reader/writer streams, and hardware events.
//! - [`worker`]: Web Worker IPC messaging bridge dispatching requests and receiving log streams.
//!
//! ## Search Tags
//! #hooks, #serial-hook, #worker-hook, #reactivity, #lifecycle

pub mod serial;
pub mod worker;
pub use serial::use_serial_controller;
pub use worker::{use_worker_controller, WorkerController};
