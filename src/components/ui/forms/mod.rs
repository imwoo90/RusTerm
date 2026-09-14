//! Form controls and interactive input components for RusTerm.
//!
//! Re-exports reusable UI controls including command input bars, custom dropdowns,
//! line ending selectors, and animated toggle switches following the design system.

mod command_input;
mod line_ending;
mod select;
mod toggle;

pub use command_input::CommandInputGroup;
#[allow(unused_imports)]
pub use line_ending::LineEndSelector;
pub use select::{CustomInputSelect, CustomSelect};
pub use toggle::ToggleSwitch;
