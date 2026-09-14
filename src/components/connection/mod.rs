//! # Connection Components Catalog (index.md)
//!
//! ## Overview
//! Aggregates serial port status displays, baud rate picker dropdowns, and serial port settings panels into cohesive UI controls for connection lifecycle management.
//!
//! ## Submodules
//! - [`baud_rate_picker`]: Dropdown selector for standard and custom serial baud rates.
//! - [`settings_dropdown`]: Configuration panel for data bits, stop bits, parity, and flow control.
//! - [`status`]: Visual connection badge showing connected, disconnected, or error states.
//!
//! ## Search Tags
//! #connection, #baud-rate, #serial-settings, #port-status

pub mod baud_rate_picker;
pub mod settings_dropdown;
pub mod status;

pub use baud_rate_picker::BaudRatePicker;
pub use settings_dropdown::SettingsDropdown;
pub use status::PortStatus;
