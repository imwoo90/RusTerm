//! # Design System UI Primitives Catalog (index.md)
//!
//! ## Overview
//! Contains shared button styles, icon controls, form inputs, feedback toasts, and console framing components ensuring visual consistency across RusTerm.
//!
//! ## Submodules
//! - [`buttons`] (`buttons.rs`): Standardized button controls, icon buttons, and interactive triggers.
//! - [`console`] (`console.rs`): Styling containers and framing wrappers for terminal and console views.
//! - [`feedback`] (`feedback.rs`): Toast alerts, status indicators, and user notification overlays.
//! - [`forms`] (`forms/mod.rs`): Reusable interactive form inputs, selectors, command bars, and toggles.
//!
//! ## Search Tags
//! #ui-primitives, #design-system, #buttons, #forms, #feedback

pub mod buttons;
pub mod console;
pub mod feedback;
pub mod forms;

pub use buttons::*;
pub use feedback::*;
pub use forms::*;
