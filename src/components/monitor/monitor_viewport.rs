//! Virtual scrolling console viewport component.
//!
//! Renders virtualized log rows within a dynamic translation container,
//! handles console keyboard event passthrough to serial ports, and tracks scroll sentinels.

use crate::components::monitor::monitor_log_line::MonitorLogLine;
use crate::config::{CONSOLE_BOTTOM_PADDING, CONSOLE_TOP_PADDING};
use crate::hooks::WorkerController;
use crate::state::{AppState, Highlight, LineEnding};
use crate::utils::serial;
use dioxus::prelude::*;
use js_sys::Uint8Array;

fn key_to_bytes(key: Key, ending: LineEnding) -> Option<Vec<u8>> {
    match key {
        Key::Character(c) => Some(c.into_bytes()),
        Key::Enter => match ending {
            LineEnding::NL => Some(vec![b'\n']),
            LineEnding::CR => Some(vec![b'\r']),
            LineEnding::NLCR => Some(vec![b'\r', b'\n']),
            LineEnding::None => Some(vec![b'\r']),
        },
        Key::Backspace => Some(vec![0x08]),
        Key::Tab => Some(vec![0x09]),
        Key::Escape => Some(vec![0x1B]),
        _ => None,
    }
}

fn handle_viewport_keydown(evt: KeyboardEvent, state: &AppState, bridge: &WorkerController) {
    let modifiers = evt.modifiers();
    if modifiers.contains(Modifiers::CONTROL)
        || modifiers.contains(Modifiers::ALT)
        || modifiers.contains(Modifiers::META)
    {
        return;
    }

    let ending = *state.serial.tx_line_ending.peek();
    let data = match key_to_bytes(evt.key(), ending) {
        Some(d) => d,
        None => return,
    };

    let port = state.conn.port.peek().as_ref().cloned();
    let local_echo = *state.serial.tx_local_echo.peek();
    let bridge_clone = bridge.clone();

    spawn(async move {
        if let Some(p) = port {
            if serial::send_data(&p, &data).await.is_ok() && local_echo {
                let array = Uint8Array::from(data.as_slice());
                bridge_clone.append_chunk(array, false);
            }
        }
    });
}

#[component]
fn ViewportLogList(
    offset_top: f64,
    highlights: Vec<Highlight>,
    show_highlights: bool,
    active_line: Option<String>,
) -> Element {
    let state = use_context::<AppState>();
    let visible_logs = state.log.visible_logs;
    let total_lines = state.log.total_lines;

    let logs = visible_logs.read();
    let is_at_bottom = logs
        .last()
        .map(|(idx, _)| *idx + 1 == total_lines())
        .unwrap_or(total_lines() == 0);

    rsx! {
        div {
            style: "position: absolute; top: 0; left: 0; right: 0; transform: translateY({offset_top}px); padding: {CONSOLE_TOP_PADDING}px 0 {CONSOLE_BOTTOM_PADDING}px 0; pointer-events: auto; min-width: 100%; width: max-content;",
            for (line_idx, text) in logs.iter() {
                MonitorLogLine {
                    key: "{line_idx}",
                    text: text.clone(),
                    highlights: highlights.clone(),
                    show_highlights,
                }
            }
            if is_at_bottom {
                if let Some(text) = active_line {
                    MonitorLogLine {
                        key: "{0}",
                        text: text.clone(),
                        highlights: highlights.clone(),
                        show_highlights: false,
                    }
                }
            }
        }
    }
}

#[component]
pub fn MonitorViewport(
    total_height: f64,
    offset_top: f64,
    onmounted_console: EventHandler<MountedEvent>,
    onscroll: EventHandler<ScrollEvent>,
    onmounted_sentinel: EventHandler<MountedEvent>,
) -> Element {
    let state = use_context::<AppState>();
    let bridge = crate::hooks::use_worker_controller();
    let visible_logs = state.log.visible_logs;
    let total_lines = state.log.total_lines;

    let highlights = (state.log.highlights)().clone();
    let show_highlights = (state.ui.show_highlights)();
    let active_line = (state.log.active_line)();

    rsx! {
        div {
            class: "flex-1 overflow-y-auto font-mono text-xs md:text-sm leading-[20px] scrollbar-custom relative",
            style: "overflow-anchor: none;",
            id: "console-output",
            tabindex: "0",
            onkeydown: move |evt| handle_viewport_keydown(evt, &state, &bridge),
            onmounted: move |evt| onmounted_console.call(evt),
            onscroll: move |evt| onscroll.call(evt),

            div { style: "height: {total_height}px; width: 100%; position: absolute; top: 0; left: 0; pointer-events: none;" }

            ViewportLogList {
                offset_top,
                highlights,
                show_highlights,
                active_line,
            }

            if visible_logs.read().is_empty() && total_lines() > 0 {
                div { class: "text-gray-500 animate-pulse text-[12px] px-4", "Loading buffer..." }
            }
            div {
                style: "position: absolute; top: {total_height}px; height: 1px; width: 100%; pointer-events: none;",
                onmounted: move |evt| onmounted_sentinel.call(evt),
            }
        }
    }
}
