//! # Virtual Scroll Calculation Hook
//!
//! ## Overview
//! Maintains state for the virtual scrolling window, computing visible start/end indices and overscan buffer margins based on scroll offsets.
//!
//! ## Search Tags
//! #virtual-scroll, #window-indices, #overscan, #scroll-offset

use crate::components::monitor::hooks::data_request::use_data_request;
use crate::components::monitor::utils::layout_utils::{
    calculate_scroll_state, calculate_virtual_metrics, use_auto_scroller, use_window_resize,
};
use crate::config::{line_height_from_font, BOTTOM_BUFFER_EXTRA, TOP_BUFFER};
use crate::state::AppState;
use crate::utils::calculate_window_size;
use dioxus::prelude::*;
use std::rc::Rc;

pub struct VirtualScroll {
    pub total_height: f64,
    pub offset_top: f64,
    pub console_handle: Signal<Option<Rc<MountedData>>>,
    pub sentinel_handle: Signal<Option<Rc<MountedData>>>,
    pub scroll_task: Resource<()>,
}

fn use_height_observer(
    console_handle: Signal<Option<Rc<MountedData>>>,
    mut console_height: Signal<f64>,
) {
    use_resource(move || {
        let handle = console_handle();
        async move {
            if let Some(handle) = handle {
                if let Ok(rect) = handle.get_client_rect().await {
                    console_height.set(rect.height());
                }
            }
        }
    });
}

fn use_scroll_observer(
    console_handle: Signal<Option<Rc<MountedData>>>,
    console_height: Signal<f64>,
    total_height: f64,
    scale_factor: f64,
    line_height: f64,
    mut start_index: Signal<usize>,
    state: AppState,
) -> Resource<()> {
    use_resource(move || {
        let handle = console_handle.peek().as_ref().cloned();
        let total_lines = (state.log.total_lines)();
        let current_height = *console_height.read();
        async move {
            if let Some(handle) = handle {
                if let Ok(offset) = handle.get_scroll_offset().await {
                    let (new_index, is_at_bottom) = calculate_scroll_state(
                        offset.y,
                        current_height,
                        total_lines,
                        scale_factor,
                        total_height,
                        line_height,
                    );
                    if (start_index)() != new_index {
                        start_index.set(new_index);
                    }
                    if (state.ui.autoscroll)() != is_at_bottom {
                        state.ui.set_autoscroll(is_at_bottom);
                    }
                }
            }
        }
    })
}

pub fn use_virtual_scroll() -> VirtualScroll {
    let state = use_context::<AppState>();

    let start_index = use_signal(|| 0usize);
    let console_height = use_signal(|| 600.0);

    let console_handle = use_signal(|| None::<Rc<MountedData>>);
    let sentinel_handle = use_signal(|| None::<Rc<MountedData>>);

    let total_lines = state.log.total_lines;
    let font_size = *state.ui.font_size.read();
    let line_height = line_height_from_font(font_size);

    let window_size = calculate_window_size(
        console_height(),
        line_height,
        TOP_BUFFER + BOTTOM_BUFFER_EXTRA,
    );

    use_window_resize(console_height, state.ui.autoscroll, sentinel_handle);
    use_data_request(start_index, window_size, total_lines);
    use_auto_scroller(state.ui.autoscroll, total_lines, sentinel_handle);

    let (total_height, offset_top, scale_factor) =
        calculate_virtual_metrics(total_lines(), start_index(), console_height(), line_height);

    use_height_observer(console_handle, console_height);

    let scroll_task = use_scroll_observer(
        console_handle,
        console_height,
        total_height,
        scale_factor,
        line_height,
        start_index,
        state,
    );

    VirtualScroll {
        total_height,
        offset_top,
        console_handle,
        sentinel_handle,
        scroll_task,
    }
}
