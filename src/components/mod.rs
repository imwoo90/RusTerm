//! # Root Components Catalog (index.md)
//!
//! ## Overview
//! Organizes top-level UI architecture into modular sub-packages including header, connection toolbar, terminal views, monitor viewports, and reusable design-system controls.
//!
//! ## Submodules
//! - [`connection`]: Serial port selection dropdowns, status indicators, and baud rate pickers.
//! - [`connection_control`]: Top-level toolbar orchestrating serial connection lifecycles and simulations.
//! - [`header`]: Application top bar displaying branding, mode toggle, and connection controls.
//! - [`monitor`]: High-performance virtualized serial monitor view, search bars, and macro buttons.
//! - [`terminal`]: Full VT100/ANSI terminal emulation view integrated with xterm.js.
//! - [`rust_term`]: Top-level container component rendering either Monitor or Terminal views.
//! - [`ui`]: Reusable design-system UI primitives, form controls, buttons, and feedback toasts.
//!
//! ## Search Tags
//! #components, #ui, #layout, #architecture, #viewports

pub mod connection;
pub mod connection_control;
pub mod header;
pub mod monitor;
pub mod terminal;

pub mod rust_term;
pub mod ui;
