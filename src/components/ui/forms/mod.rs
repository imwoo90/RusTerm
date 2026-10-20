//! # Form Controls Catalog (index.md)
//!
//! ## Overview
//! Re-exports reusable UI controls including command input bars, custom dropdowns, line ending selectors, and animated toggle switches following the design system.
//!
//! ## Submodules
//! - [`command_input`] (`command_input.rs`): Text entry bar for typing and transmitting serial commands with history.
//! - [`select`] (`select.rs`): Styled select dropdowns, line ending selectors, and boolean toggle switches.
//!
//! ## Search Tags
//! #forms, #inputs, #controls, #dropdowns, #toggles

mod command_input;
mod select;

pub use command_input::CommandInputGroup;
#[allow(unused_imports)]
pub use select::{CustomInputSelect, CustomSelect, LineEndSelector, ToggleSwitch};
