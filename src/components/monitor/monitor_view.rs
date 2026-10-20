//! # Monitor View Container, Header & Control Toolbar
//!
//! ## Overview
//! Combines the virtualized scroll viewport, monitor header (`MonitorHeader`), control toolbar (`MonitorToolbar`),
//! search filter bar, and command transmit strip into a unified serial monitor interface (`Monitor`).
//!
//! ## Search Tags
//! #monitor-view, #monitor-header, #monitor-toolbar, #container, #serial-monitor

use crate::components::monitor::hooks::{use_search_sync, use_settings_sync, use_virtual_scroll};
use crate::components::monitor::monitor_viewport::MonitorViewport;
use crate::components::monitor::{HighlightButton, SearchBar, TransmitBar};
use crate::components::ui::buttons::ResumeScrollButton;
use crate::components::ui::console::{
    ConsoleFrame, ConsoleSeparator, ConsoleToggleButton, UnifiedConsoleToolbar,
};
use crate::hooks::use_worker_controller;
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn MonitorToolbar() -> Element {
    let highlight_open = use_signal(|| false);

    rsx! {
        div {
            class: "shrink-0 p-2 bg-background-dark relative",
            class: if highlight_open() { "z-60" } else { "z-40" },
            div { class: "flex gap-2 h-10 items-stretch min-w-[600px]",
                HighlightButton { is_open: highlight_open }
                SearchBar {}
                div { class: "w-px bg-[#2a2e33] my-2 mx-1" }
                TransmitBar {}
            }
        }
    }
}

#[component]
pub fn MonitorHeader(
    autoscroll: bool,
    count: usize,
    onexport: EventHandler<MouseEvent>,
    onclear: EventHandler<MouseEvent>,
    ontoggle_autoscroll: EventHandler<MouseEvent>,
) -> Element {
    let state = use_context::<AppState>();

    rsx! {
        UnifiedConsoleToolbar {
            left: rsx! {
                span { class: "text-[10px] text-gray-500 font-mono hidden sm:block", "[ LINES: {count} / OPFS ENABLED ]" }
                ConsoleSeparator {}
                div { class: "flex items-center gap-1",
                    ConsoleToggleButton {
                        icon: "schedule",
                        title: "Toggle Timestamps",
                        active: (state.ui.show_timestamps)(),
                        onclick: move |_| state.ui.toggle_timestamps(),
                    }
                    ConsoleToggleButton {
                        icon: "hexagon",
                        title: "Toggle Hex View",
                        active: (state.ui.is_hex_view)(),
                        onclick: move |_| state.ui.toggle_hex_view(),
                    }
                }
            },
            font_size: state.ui.font_size,
            is_autoscroll: autoscroll,
            is_tracking_interactive: true,
            on_toggle_autoscroll: move |evt| ontoggle_autoscroll.call(evt),
            on_clear: move |evt| onclear.call(evt),
            on_export: move |evt| onexport.call(evt),
            min_font_size: 8,
            max_font_size: 36,
        }
    }
}

#[component]
pub fn Monitor() -> Element {
    let state = use_context::<AppState>();
    let bridge = use_worker_controller();
    let mut vs = use_virtual_scroll();

    use_settings_sync(bridge);
    use_search_sync(bridge);

    rsx! {
        ConsoleFrame {
            MonitorHeader {
                autoscroll: (state.ui.autoscroll)(),
                count: (state.log.total_lines)(),
                onexport: move |_| bridge.export((state.ui.show_timestamps)()),
                onclear: move |_| {
                    bridge.clear();
                    state.log.clear();
                    state.success("Logs Cleared");
                },
                ontoggle_autoscroll: move |_| state.ui.toggle_autoscroll(),
            }

            MonitorViewport {
                total_height: vs.total_height,
                offset_top: vs.offset_top,
                onmounted_console: move |evt: MountedEvent| {
                    vs.console_handle.set(Some(evt.data()));
                },
                onscroll: move |_: ScrollEvent| {
                    vs.scroll_task.restart();
                },
                onmounted_sentinel: move |evt: MountedEvent| vs.sentinel_handle.set(Some(evt.data())),
            }

            if !(state.ui.autoscroll)() {
                ResumeScrollButton {
                    onclick: move |_| {
                        if let Some(el) = web_sys::window()
                            .and_then(|win| win.document())
                            .and_then(|doc| doc.get_element_by_id("console-output"))
                        {
                            el.set_scroll_top(el.scroll_height());
                        }
                    },
                }
            }
        }
    }
}
