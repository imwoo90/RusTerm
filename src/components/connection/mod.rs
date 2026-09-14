//! Connection controls and port configuration sub-module (mod.rs = index.md).
//!
//! Aggregates serial port status displays, baud rate picker dropdowns, and serial port settings panels
//! into cohesive UI controls for connection lifecycle management.

pub mod baud_rate_picker;
pub mod settings_dropdown;
pub mod status;

pub use baud_rate_picker::BaudRatePicker;
pub use settings_dropdown::SettingsDropdown;
pub use status::PortStatus;
