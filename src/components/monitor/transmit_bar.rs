//! # Serial Transmit Bar
//!
//! ## Overview
//! Renders the bottom command entry bar for sending ASCII text, escape sequences, or raw hexadecimal bytes directly to connected serial devices.
//!
//! ## Search Tags
//! #transmit-bar, #send-command, #hex-transmit, #serial-tx

use crate::components::ui::forms::CommandInputGroup;
use crate::state::{AppState, LineEnding};
use crate::utils::serial;
use crate::utils::{parse_hex_string, CommandHistory};
use dioxus::prelude::*;
use futures_util::StreamExt;
use js_sys::Uint8Array;

async fn execute_send(
    text: String,
    is_hex: bool,
    ending: LineEnding,
    local_echo: bool,
    port: Option<web_sys::SerialPort>,
    bridge: crate::hooks::WorkerController,
    mut history: Signal<CommandHistory>,
    mut history_index: Signal<Option<usize>>,
    mut input_value: Signal<String>,
) {
    if text.is_empty() && matches!(ending, LineEnding::None) {
        return;
    }

    let mut data = if is_hex {
        if text.is_empty() {
            Vec::new()
        } else {
            match parse_hex_string(&text) {
                Ok(d) => d,
                Err(e) => {
                    if let Some(w) = web_sys::window() {
                        let _ = w.alert_with_message(&format!("Hex Error: {}", e));
                    }
                    return;
                }
            }
        }
    } else {
        text.clone().into_bytes()
    };

    match ending {
        LineEnding::NL => data.push(b'\n'),
        LineEnding::CR => data.push(b'\r'),
        LineEnding::NLCR => {
            data.push(b'\r');
            data.push(b'\n');
        }
        _ => {}
    }

    if let Some(conn_port) = port {
        if serial::send_data(&conn_port, &data).await.is_ok() {
            if local_echo {
                let array = Uint8Array::from(data.as_slice());
                bridge.append_chunk(array, false);
            }
            input_value.set(String::new());
            if !text.is_empty() {
                history.write().add(text);
            }
            history_index.set(None);
        }
    }
}

fn handle_history_key(
    key: &Key,
    history: &Signal<CommandHistory>,
    history_index: &mut Signal<Option<usize>>,
    input_value: &mut Signal<String>,
) {
    match key {
        Key::ArrowUp => {
            let h = history.read();
            if h.len() > 0 {
                let idx = history_index()
                    .map(|i| if i > 0 { i - 1 } else { 0 })
                    .unwrap_or(h.len() - 1);
                history_index.set(Some(idx));
                if let Some(c) = h.get_at(idx) {
                    input_value.set(c.clone());
                }
            }
        }
        Key::ArrowDown => {
            if let Some(i) = history_index() {
                let h = history.read();
                if i + 1 >= h.len() {
                    history_index.set(None);
                    input_value.set(String::new());
                } else {
                    history_index.set(Some(i + 1));
                    if let Some(c) = h.get_at(i + 1) {
                        input_value.set(c.clone());
                    }
                }
            }
        }
        _ => {}
    }
}

#[component]
fn SendButton(onclick: EventHandler<()>) -> Element {
    rsx! {
        button {
            class: "h-full aspect-square bg-primary text-surface rounded-lg flex items-center justify-center hover:bg-white transition-all hover:shadow-[0_0_15px_rgba(255,255,255,0.4)] active:scale-95 group",
            onclick: move |_| onclick.call(()),
            span { class: "material-symbols-outlined text-[20px] group-hover:rotate-45 transition-transform duration-300",
                "send"
            }
        }
    }
}

#[component]
pub fn TransmitBar() -> Element {
    let state = use_context::<AppState>();
    let mut input_value = use_signal(String::new);
    let history = use_signal(CommandHistory::load);
    let mut history_index = use_signal(|| None::<usize>);
    let is_hex_input = use_signal(|| false);

    let bridge = crate::hooks::use_worker_controller();

    let send_task = use_coroutine(move |mut rx| {
        let b = bridge.clone();
        async move {
            while rx.next().await.is_some() {
                let text = input_value();
                let ending = *state.serial.tx_line_ending.peek();
                let is_hex = is_hex_input();
                let local_echo = *state.serial.tx_local_echo.peek();
                let port = state.conn.port.peek().as_ref().cloned();
                execute_send(
                    text, is_hex, ending, local_echo, port,
                    b.clone(), history, history_index, input_value,
                ).await;
            }
        }
    });

    rsx! {
        div { class: "flex-1 relative flex gap-2 min-w-0",
            CommandInputGroup {
                value: input_value,
                is_hex: is_hex_input,
                line_ending: state.serial.tx_line_ending,
                echo: Some(state.serial.tx_local_echo),
                placeholder: "Send command...",
                id: Some("transmit-input"),
                on_submit: move |_| send_task.send(()),
                on_keydown: move |evt: KeyboardEvent| {
                    let key = evt.key();
                    if matches!(key, Key::ArrowUp | Key::ArrowDown) {
                        handle_history_key(&key, &history, &mut history_index, &mut input_value);
                        evt.prevent_default();
                    }
                },
            }

            SendButton { onclick: move |_| send_task.send(()) }
        }
    }
}
