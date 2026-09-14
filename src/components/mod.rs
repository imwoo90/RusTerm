//! Root components module catalog (mod.rs = index.md).
//!
//! Organizes top-level UI architecture into modular sub-packages including header, connection toolbar,
//! terminal views, monitor viewports, and reusable design-system controls.

pub mod connection;
pub mod connection_control;
pub mod header;
pub mod monitor;
pub mod terminal;

pub mod rust_term;
pub mod ui;
