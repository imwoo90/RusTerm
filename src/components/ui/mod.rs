//! Reusable design system UI components catalog (mod.rs = index.md).
//!
//! Contains shared button styles, icon controls, form inputs, feedback toasts, and console framing
//! components ensuring visual consistency across RusTerm.

pub mod buttons;
pub mod console;
pub mod feedback;
pub mod forms;

pub use buttons::*;
pub use feedback::*;
pub use forms::*;
