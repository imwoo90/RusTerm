use crate::components::console::log_line::LogLine;
use crate::config::{CONSOLE_BOTTOM_PADDING, CONSOLE_TOP_PADDING};
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn LogViewport(
    total_height: f64,
    offset_top: f64,
    onmounted_console: EventHandler<MountedEvent>,
    onscroll: EventHandler<ScrollEvent>,
    onmounted_sentinel: EventHandler<MountedEvent>,
) -> Element {
    let state = use_context::<AppState>();
    let visible_logs = state.log.visible_logs;
    let total_lines = state.log.total_lines;

    rsx! {
        div {
            class: "flex-1 overflow-y-auto font-mono text-xs md:text-sm leading-[20px] scrollbar-custom relative",
            style: "overflow-anchor: none;",
            id: "console-output",
            onmounted: move |evt| onmounted_console.call(evt),
            onscroll: move |evt| onscroll.call(evt),

            // Virtual Scroll Spacer & Content
            div { style: "height: {total_height}px; width: 100%; position: absolute; top: 0; left: 0; pointer-events: none;" }
            div { style: "position: absolute; top: 0; left: 0; right: 0; transform: translateY({offset_top}px); padding: {CONSOLE_TOP_PADDING}px 1rem {CONSOLE_BOTTOM_PADDING}px 1rem; pointer-events: auto; min-width: 100%; width: max-content;",
                {
                    let highlights = (state.log.highlights)().clone();
                    let show_timestamps = (state.ui.show_timestamps)();
                    let show_highlights = (state.ui.show_highlights)();
                    let active_line = (state.log.active_line)();
                    let logs = visible_logs.read();
                    let is_at_bottom = logs.last().map(|(idx, _)| *idx + 1 == total_lines()).unwrap_or(total_lines() == 0);

                    rsx! {
                        for (line_idx, text) in logs.iter() {
                            LogLine {
                                key: "{line_idx}",
                                text: text.clone(),
                                highlights: highlights.clone(),
                                show_timestamps,
                                show_highlights,
                            }
                        }
                        if is_at_bottom {
                             if let Some(text) = active_line {
                                 LogLine {
                                    key: "{0}",
                                    text: text.clone(),
                                    highlights: highlights.clone(),
                                    show_timestamps,
                                    show_highlights: false, // Maybe don't highlight active line to avoid flicker?
                                 }
                             }
                        }
                    }
                }
            }

            // Loading & Sentinel
            if visible_logs.read().is_empty() && total_lines() > 0 {
                div { class: "text-gray-500 animate-pulse text-[12px] px-4",
                    "Loading buffer..."
                }
            }
            div {
                style: "position: absolute; top: {total_height}px; height: 1px; width: 100%; pointer-events: none;",
                onmounted: move |evt| onmounted_sentinel.call(evt),
            }
        }
    }
}
