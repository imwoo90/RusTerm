//! # Monitor Subsystem Catalog (index.md)
//!
//! ## Overview
//! Centralizes high-performance virtualized log viewing, search filter bars, macro strips, highlight tagging rules, and streaming line formatters.
//!
//! ## Submodules
//! - [`highlight`] (`highlight.rs`): Text pattern highlighting rules, tag badges, and custom color accents.
//! - [`hooks`] (`hooks.rs`): Reactive hooks managing virtualized scrolling, buffer fetch windows, and effects.
//! - [`macro_bar`] (`macro_bar/mod.rs`): Quick-access macro strip for triggering preconfigured serial command sequences.
//! - [`monitor_view`] (`monitor_view.rs`): Container view combining viewport, header, toolbar, macro strip, and search bars.
//! - [`monitor_viewport`] (`monitor_viewport.rs`): High-speed virtualized scroll viewport and ANSI log line renderer.
//! - [`search_bar`] (`search_bar.rs`): Incremental regex and text search input bar with match navigation.
//! - [`transmit_bar`] (`transmit_bar.rs`): Bottom transmit bar for sending raw text and hex bytes to serial ports.
//! - [`utils`] (`utils.rs`): Layout calculation math and visual styling helpers for monitor components.
//!
//! ## Search Tags
//! #monitor, #virtual-scroll, #log-viewer, #highlighting, #macros

pub mod highlight;
pub mod hooks;
pub mod macro_bar;
pub mod monitor_view;
pub mod monitor_viewport;
pub mod search_bar;
pub mod transmit_bar;
pub mod utils;

pub use highlight::HighlightButton;
pub use macro_bar::MacroBar;
pub use monitor_view::{Monitor, MonitorToolbar};
pub use search_bar::SearchBar;
pub use transmit_bar::TransmitBar;
