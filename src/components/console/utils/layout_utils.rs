use crate::config::HEADER_OFFSET;
use dioxus::prelude::*;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

/// Hook to handle window resize events and adjust console height
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

                // Only update if height changed significantly
                // Using peek() here ensures this effect runs ONLY ONCE on mount
                if (*console_height.peek() - new_height).abs() > 0.1 {
                    console_height.set(new_height);
                }
            }
        };

        update(); // Initial execution
        let onresize = Closure::wrap(Box::new(update) as Box<dyn FnMut()>);
        window.set_onresize(Some(onresize.as_ref().unchecked_ref()));
        onresize.forget();
    });
    // Use use_resource to handle scrolling reactively.
    // This is more efficient as it automatically cancels previous tasks if a new change occurs
    // while we are waiting (TimeoutFuture).
    let _ = use_resource(move || async move {
        console_height(); // Subscribe to height changes
        let auto = autoscroll(); // Subscribe to autoscroll changes

        if auto {
            // Wait a tick for the DOM to update with new height
            gloo_timers::future::TimeoutFuture::new(10).await;
            if let Some(s) = sentinel.peek().as_ref() {
                let _ = s.scroll_to(ScrollBehavior::Instant).await;
            }
        }
    });
}

/// Hook to manage auto-scroll functionality
pub fn use_auto_scroller(
    autoscroll: Signal<bool>,
    total_lines: Signal<usize>,
    _sentinel: Signal<Option<Rc<MountedData>>>, // Sentinel no longer needed
) {
    use_effect(move || {
        total_lines(); // React to changes
        if (autoscroll)() {
            // Use plain JS to set scrollTop ONLY, preserving scrollLeft.
            // Dioxus visible/scrollTo APIs often mess with X-axis.
            // Element ID is "console-output"
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Some(el) = document.get_element_by_id("console-output") {
                        // scrollTop = scrollHeight
                        let scroll_height = el.scroll_height();
                        el.set_scroll_top(scroll_height);
                    }
                }
            }
        }
    });
}

/// Helper to calculate new scroll state (start_index and autoscroll)
/// Returns (new_start_index, should_autoscroll)
/// Helper to calculate new scroll state (start_index and autoscroll)
/// Returns (new_start_index, should_autoscroll)
pub fn calculate_scroll_state(
    offset_y: f64,
    viewport_height: f64,
    total_lines: usize,
) -> (usize, bool) {
    use crate::config::{CONSOLE_BOTTOM_PADDING, CONSOLE_TOP_PADDING, LINE_HEIGHT, TOP_BUFFER};
    use crate::utils::calculate_start_index;

    // 1. Calculate Virtual Scroll Index
    // No scaling needed
    let new_index = calculate_start_index(offset_y, LINE_HEIGHT, TOP_BUFFER);

    // 2. Autoscroll Detection (Math-based)
    let total_content_height =
        (total_lines as f64) * LINE_HEIGHT + CONSOLE_TOP_PADDING + CONSOLE_BOTTOM_PADDING;

    // Allow small buffer (e.g. 10px) for precision
    let is_at_bottom = if total_content_height <= viewport_height {
        true
    } else {
        // Check if we are at the bottom of the container
        offset_y + viewport_height >= total_content_height - 10.0
    };

    (new_index, is_at_bottom)
}

// Removed ConsoleHeader and ResumeScrollButton to separate files

/// Calculates virtual scroll metrics (total_height, scale_factor, offset_top)
pub fn calculate_virtual_metrics(total_lines: usize, start_index: usize) -> (f64, f64) {
    use crate::config::{CONSOLE_BOTTOM_PADDING, CONSOLE_TOP_PADDING, LINE_HEIGHT};

    let real_total_height =
        (total_lines as f64) * LINE_HEIGHT + CONSOLE_TOP_PADDING + CONSOLE_BOTTOM_PADDING;

    // Scaling removed as requested
    let total_height = real_total_height;

    let offset_top = (start_index as f64) * LINE_HEIGHT;

    (total_height, offset_top)
}
