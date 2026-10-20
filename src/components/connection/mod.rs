//! # Connection Components Catalog (index.md)
//!
//! ## Overview
//! Aggregates serial port status displays, baud rate picker dropdowns, and serial port settings panels into cohesive UI controls for connection lifecycle management.
//!
//! ## Submodules
//! - [`settings_dropdown`] (`settings_dropdown.rs`): Baud rate picker dropdown and configuration panel for data bits, stop bits, parity, and flow control.
//! - [`status`] (`status.rs`): Visual connection badge showing connected, disconnected, or error states.
//!
//! ## Search Tags
//! #connection, #baud-rate, #serial-settings, #port-status

pub mod settings_dropdown;
pub mod status;

pub use settings_dropdown::{BaudRatePicker, SettingsDropdown};
pub use status::PortStatus;
