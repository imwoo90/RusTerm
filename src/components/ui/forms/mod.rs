//! # Form Controls Catalog (index.md)
//!
//! ## Overview
//! Re-exports reusable UI controls including command input bars, custom dropdowns, line ending selectors, and animated toggle switches following the design system.
//!
//! ## Submodules
//! - [`command_input`]: Text entry bar for typing and transmitting serial commands with history.
//! - [`line_ending`]: Dropdown selector for carriage return and newline line terminators.
//! - [`select`]: Generic styled select dropdown component for option picking.
//! - [`toggle`]: Reusable animated boolean toggle switch control.
//!
//! ## Search Tags
//! #forms, #inputs, #controls, #dropdowns, #toggles

mod command_input;
mod line_ending;
mod select;
mod toggle;

pub use command_input::CommandInputGroup;
#[allow(unused_imports)]
pub use line_ending::LineEndSelector;
pub use select::{CustomInputSelect, CustomSelect};
pub use toggle::ToggleSwitch;
