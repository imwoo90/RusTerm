//! # Monitor Layout Math & Styling Utilities
//!
//! ## Overview
//! Provides virtual viewport dimension calculations (`calculate_virtual_metrics`, `calculate_scroll_state`),
//! auto-scroll and window resize observers, and highlight badge CSS class mappings (`get_highlight_classes`).
//!
//! ## Search Tags
//! #monitor-utils, #layout-utils, #scroll-math, #viewport-dimensions, #style, #colors

use crate::config::HEADER_OFFSET;
use dioxus::prelude::*;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

const COLOR_MAP: &[(&str, &str, &str)] = &[
    ("red", "border-red-500/30 hover:border-red-500/60", "text-red-400"),
    ("blue", "border-blue-500/30 hover:border-blue-500/60", "text-blue-400"),
    ("yellow", "border-yellow-500/30 hover:border-yellow-500/60", "text-yellow-400"),
    ("green", "border-green-500/30 hover:border-green-500/60", "text-green-400"),
    ("purple", "border-purple-500/30 hover:border-purple-500/60", "text-purple-400"),
    ("orange", "border-orange-500/30 hover:border-orange-500/60", "text-orange-400"),
    ("teal", "border-teal-500/30 hover:border-teal-500/60", "text-teal-400"),
    ("pink", "border-pink-500/30 hover:border-pink-500/60", "text-pink-400"),
    ("indigo", "border-indigo-500/30 hover:border-indigo-500/60", "text-indigo-400"),
    ("lime", "border-lime-500/30 hover:border-lime-500/60", "text-lime-400"),
    ("cyan", "border-cyan-500/30 hover:border-cyan-500/60", "text-cyan-400"),
    ("rose", "border-rose-500/30 hover:border-rose-500/60", "text-rose-400"),
    ("fuchsia", "border-fuchsia-500/30 hover:border-fuchsia-500/60", "text-fuchsia-400"),
    ("amber", "border-amber-500/30 hover:border-amber-500/60", "text-amber-400"),
    ("emerald", "border-emerald-500/30 hover:border-emerald-500/60", "text-emerald-400"),
    ("sky", "border-sky-500/30 hover:border-sky-500/60", "text-sky-400"),
    ("violet", "border-violet-500/30 hover:border-violet-500/60", "text-violet-400"),
];

/// Returns `(border_class, text_class)` for a given highlight color name.
pub fn get_highlight_classes(color: &str) -> (&'static str, &'static str) {
    COLOR_MAP
        .iter()
        .find(|(c, _, _)| *c == color)
        .map(|(_, border, text)| (*border, *text))
        .unwrap_or(("border-primary/30 hover:border-primary/60", "text-primary"))
}

/// Hook to handle window resize events and adjust console height.
pub fn use_window_resize(
    mut console_height: Signal<f64>,
    autoscroll: Signal<bool>,
    sentinel: Signal<Option<Rc<MountedData>>>,
) {
    use_effect(move || {
        let window = match web_sys::window() {
            Some(w) => w,
            None => return,
        };

        let mut update = move || {
            let window = match web_sys::window() {
                Some(w) => w,
                None => return,
            };

            if let Ok(Some(h)) = window.inner_height().map(|jv| jv.as_f64()) {
                let new_height = (h - HEADER_OFFSET).max(100.0);

                if (*console_height.peek() - new_height).abs() > 0.1 {
                    console_height.set(new_height);
                }
            }
        };

        update();
        let onresize = Closure::wrap(Box::new(update) as Box<dyn FnMut()>);
        window.set_onresize(Some(onresize.as_ref().unchecked_ref()));
        onresize.forget();
    });

    use_resource(move || async move {
        console_height();
        let auto = autoscroll();

        if auto {
            gloo_timers::future::TimeoutFuture::new(10).await;
            if let Some(s) = sentinel.peek().as_ref() {
                let _ = s.scroll_to(ScrollBehavior::Instant).await;
            }
        }
    });
}

/// Hook to manage auto-scroll functionality.
pub fn use_auto_scroller(
    autoscroll: Signal<bool>,
    total_lines: Signal<usize>,
    _sentinel: Signal<Option<Rc<MountedData>>>,
) {
    use_effect(move || {
        total_lines();
        if (autoscroll)()
            && let Some(window) = web_sys::window()
            && let Some(document) = window.document()
            && let Some(el) = document.get_element_by_id("console-output")
        {
            let scroll_height = el.scroll_height();
            el.set_scroll_top(scroll_height);
        }
    });
}

/// Computes the starting log line index from vertical scroll offset and top overscan buffer.
pub fn calculate_start_index(scroll_y: f64, line_height: f64, top_buffer: usize) -> usize {
    let raw_index = (scroll_y / line_height).floor() as usize;
    raw_index.saturating_sub(top_buffer)
}

/// Computes the number of log lines to request for the visible viewport plus overscan buffers.
pub fn calculate_window_size(viewport_height: f64, line_height: f64, buffer_size: usize) -> usize {
    let visible_lines = (viewport_height / line_height).ceil() as usize;
    visible_lines + buffer_size
}

/// Helper to calculate new scroll state (`new_start_index`, `should_autoscroll`).
pub fn calculate_scroll_state(
    offset_y: f64,
    viewport_height: f64,
    total_lines: usize,
    scale_factor: f64,
    physical_total_height: f64,
    line_height: f64,
) -> (usize, bool) {
    use crate::config::TOP_BUFFER;

    // 1. Physical Bottom Detection
    let is_at_bottom = if physical_total_height <= viewport_height {
        true
    } else {
        offset_y + viewport_height >= physical_total_height - 15.0
    };

    // 2. Calculate Start Index with Clamping
    let new_index = if is_at_bottom {
        let visible_lines = (viewport_height / line_height).ceil() as usize;
        total_lines.saturating_sub(visible_lines)
    } else {
        let logical_offset_y = offset_y * scale_factor;
        calculate_start_index(logical_offset_y, line_height, TOP_BUFFER)
    };

    (new_index, is_at_bottom)
}

/// Calculates virtual scroll metrics (`total_height`, `offset_top`, `scale_factor`).
pub fn calculate_virtual_metrics(
    total_lines: usize,
    start_index: usize,
    viewport_height: f64,
    line_height: f64,
) -> (f64, f64, f64) {
    use crate::config::{CONSOLE_BOTTOM_PADDING, CONSOLE_TOP_PADDING, VIRTUAL_SCROLL_THRESHOLD};

    let real_total_height =
        (total_lines as f64) * line_height + CONSOLE_TOP_PADDING + CONSOLE_BOTTOM_PADDING;

    let (total_height, scale_factor) = if real_total_height > VIRTUAL_SCROLL_THRESHOLD {
        let max_physical_scroll_range = VIRTUAL_SCROLL_THRESHOLD - viewport_height;
        let logical_scroll_range = real_total_height - viewport_height;

        let ratio = logical_scroll_range / max_physical_scroll_range;
        let sf = 2.0_f64.powf(ratio.log2().ceil());

        let physical_total_height = (logical_scroll_range / sf) + viewport_height;
        (physical_total_height, sf)
    } else {
        (real_total_height, 1.0)
    };

    let logical_offset_top = (start_index as f64) * line_height;
    let offset_top = (logical_offset_top / scale_factor).floor();

    (total_height, offset_top, scale_factor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_math() {
        let line_height = 20.0;
        let top_buffer = 5;

        assert_eq!(calculate_start_index(0.0, line_height, top_buffer), 0);
        assert_eq!(calculate_start_index(100.0, line_height, top_buffer), 0);
        assert_eq!(calculate_start_index(120.0, line_height, top_buffer), 1);
        assert_eq!(calculate_window_size(600.0, line_height, 15), 45);
        assert_eq!(calculate_window_size(610.0, line_height, 15), 46);
    }
}
