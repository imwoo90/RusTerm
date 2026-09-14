//! # Global Configuration Parameters
//!
//! ## Overview
//! Centralizes line buffer limits, typography settings, padding dimensions, and virtual scrolling thresholds to guarantee consistent rendering and optimal memory usage across the application.
//!
//! ## Search Tags
//! #config, #limits, #typography, #virtual-scroll, #constants

/// --- Networking & Buffer Config ---
pub const READ_BUFFER_SIZE: usize = 64 * 1024;
pub const EXPORT_CHUNK_SIZE: u64 = 64 * 1024;
pub const MAX_LINE_BYTES: usize = 256;
pub const HEX_VIEW_BYTES: usize = 16;

/// --- UI Timing & Intervals ---
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub const TOAST_DURATION_MS: u32 = 3000;
pub const WORKER_UPDATE_INTERVAL_MS: u32 = 16;
pub const APP_SUBTITLE: &str = "RusTerm v3.3.0";

/// --- Layout & Virtual Scroll ---
pub const HEADER_OFFSET: f64 = 150.0;
pub const TOP_BUFFER: usize = 10;
pub const BOTTOM_BUFFER_EXTRA: usize = 40;
pub const CONSOLE_TOP_PADDING: f64 = 8.0; // 0.5rem
pub const CONSOLE_BOTTOM_PADDING: f64 = 20.0;
pub const VIRTUAL_SCROLL_THRESHOLD: f64 = 10_000_000.0;

/// Calculate line height from font size (font_size * 1.4 for readable spacing)
pub fn line_height_from_font(font_size: u32) -> f64 {
    (font_size as f64) * 1.4
}

pub const HIGHLIGHT_COLORS: &[&str] = &[
    "red", "blue", "yellow", "green", "purple", "orange", "teal", "pink", "indigo", "lime", "cyan",
    "rose", "fuchsia", "amber", "emerald", "sky", "violet",
];
