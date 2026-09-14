//! # Monitor Subsystem Catalog (index.md)
//!
//! ## Overview
//! Centralizes high-performance virtualized log viewing, search filter bars, macro strips, highlight tagging rules, and streaming line formatters.
//!
//! ## Submodules
//! - [`highlight`]: Text pattern highlighting rules, tag badges, and custom color accents.
//! - [`hooks`]: Reactive hooks managing virtualized scrolling, buffer fetch windows, and effects.
//! - [`macro_bar`]: Quick-access macro strip for triggering preconfigured serial command sequences.
//! - [`monitor_header`]: Viewport top bar with filter toggles, counter stats, and clear actions.
//! - [`monitor_log_line`]: Individual virtualized log line renderer with timestamps and ANSI styles.
//! - [`monitor_toolbar`]: Action bar managing display modes, line wrapping, and autoscroll states.
//! - [`monitor_view`]: Container view combining viewport, toolbar, macro strip, and search bars.
//! - [`monitor_viewport`]: High-speed virtualized scroll viewport rendering visible log slices.
//! - [`search_bar`]: Incremental regex and text search input bar with match navigation.
//! - [`transmit_bar`]: Bottom transmit bar for sending raw text and hex bytes to serial ports.
//! - [`utils`]: Layout calculation math and visual styling helpers for monitor components.
//!
//! ## Search Tags
//! #monitor, #virtual-scroll, #log-viewer, #highlighting, #macros

pub mod highlight;
pub mod hooks;
pub mod macro_bar;
pub mod monitor_header;
pub mod monitor_log_line;
pub mod monitor_toolbar;
pub mod monitor_view;
pub mod monitor_viewport;
pub mod search_bar;
pub mod transmit_bar;
pub mod utils;

pub use highlight::HighlightButton;
pub use macro_bar::MacroBar;
pub use monitor_toolbar::MonitorToolbar;
pub use monitor_view::Monitor;
pub use search_bar::SearchBar;
pub use transmit_bar::TransmitBar;
