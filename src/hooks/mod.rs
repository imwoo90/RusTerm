//! Reactive hooks catalog for RusTerm (mod.rs = index.md).
//!
//! Re-exports domain controllers and async workers managing Web Serial connections,
//! Web Worker messaging bridges, and reactive component lifecycles.

pub mod serial;
pub mod worker;
pub use serial::use_serial_controller;
pub use worker::{use_worker_controller, WorkerController};
