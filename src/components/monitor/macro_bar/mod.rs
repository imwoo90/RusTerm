//! # Macro Bar Catalog (index.md)
//!
//! ## Overview
//! Provides a persistent horizontal strip of quick-access macro buttons for triggering pre-configured serial commands, adding macros, and editing entries.
//!
//! ## Submodules
//! - [`context_menu`]: Context menu for editing, reordering, or deleting existing macro buttons.
//! - [`form_modal`]: Modal dialog for creating and editing macro titles, commands, and keybindings.
//!
//! ## Search Tags
//! #macro-bar, #quick-commands, #macros, #shortcuts

mod context_menu;
mod form_modal;

use context_menu::MacroContextMenu;
use form_modal::MacroFormModal;

use crate::state::{AppState, LineEnding};
use crate::utils::serial;
use crate::utils::{parse_hex_string, MacroStorage};
use dioxus::prelude::*;

async fn execute_macro_send(
    cmd: String,
    is_hex: bool,
    ending: LineEnding,
    port: Option<web_sys::SerialPort>,
    local_echo: bool,
    bridge: crate::hooks::WorkerController,
    state: AppState,
) {
    let mut data = if is_hex {
        match parse_hex_string(&cmd) {
            Ok(d) => d,
            Err(e) => {
                state.error(&format!("Macro Hex Error: {}", e));
                return;
            }
        }
    } else {
        cmd.into_bytes()
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
        if serial::send_data(&conn_port, &data).await.is_ok() && local_echo {
            let array = js_sys::Uint8Array::from(data.as_slice());
            bridge.append_chunk(array, false);
        }
    } else {
        state.warning("Serial port is not connected");
    }
}

#[component]
fn MacroButton(
    label: String,
    title: String,
    onclick: EventHandler<()>,
    oncontextmenu: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        button {
            class: "shrink-0 px-3 py-1 bg-[#2a2e33] hover:bg-primary hover:text-white rounded text-xs font-mono transition-colors border border-gray-700 select-none whitespace-nowrap",
            onclick: move |_| onclick.call(()),
            oncontextmenu: move |evt| oncontextmenu.call(evt),
            title: "{title}",
            "{label}"
        }
    }
}

#[component]
fn AddMacroButton(onclick: EventHandler<()>) -> Element {
    rsx! {
        button {
            class: "shrink-0 w-6 h-6 flex items-center justify-center bg-[#1a1c1e] text-gray-400 hover:text-white rounded text-xs border border-dashed border-gray-700 hover:border-gray-500 transition-colors",
            onclick: move |_| onclick.call(()),
            title: "Add Macro",
            "+"
        }
    }
}

#[component]
fn GitHubLink() -> Element {
    rsx! {
        div { class: "shrink-0 flex items-center gap-4 ml-auto px-2",
            a {
                class: "text-gray-500 hover:text-primary transition-colors flex items-center gap-1.5 group text-[11px]",
                href: "https://github.com/imwoo90/rusterm",
                target: "_blank",
                span { class: "material-symbols-outlined text-[14px]", "code" }
                span { "GitHub" }
            }
        }
    }
}

#[component]
fn MacroList(
    storage: Signal<MacroStorage>,
    mut context_menu: Signal<Option<(u64, i32, i32)>>,
    on_send: EventHandler<(String, bool, LineEnding)>,
    on_add: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "flex gap-2 flex-1 items-center",
            for item in storage.read().get_items() {
                {
                    let cmd = item.command.clone();
                    let is_hex = item.is_hex;
                    let line_ending = item.line_ending;
                    let id = item.id;
                    rsx! {
                        MacroButton {
                            key: "{id}",
                            label: item.label.clone(),
                            title: item.command.clone(),
                            onclick: move |_| on_send.call((cmd.clone(), is_hex, line_ending)),
                            oncontextmenu: move |evt: MouseEvent| {
                                evt.prevent_default();
                                let coords = evt.client_coordinates();
                                context_menu.set(Some((id, coords.x as i32, coords.y as i32)));
                            },
                        }
                    }
                }
            }
            AddMacroButton { onclick: move |_| on_add.call(()) }
        }
    }
}

#[component]
pub fn MacroBar() -> Element {
    let state = use_context::<AppState>();
    let storage = use_signal(MacroStorage::load);
    let mut show_form = use_signal(|| false);
    let bridge = crate::hooks::use_worker_controller();

    let mut new_label = use_signal(String::new);
    let mut new_cmd = use_signal(String::new);
    let mut new_hex = use_signal(|| false);
    let mut new_ending = use_signal(|| LineEnding::None);
    let mut editing_id = use_signal(|| None::<u64>);
    let context_menu = use_signal(|| None::<(u64, i32, i32)>);

    let on_send = move |(cmd, is_hex, ending): (String, bool, LineEnding)| {
        let port = state.conn.port.peek().as_ref().cloned();
        let local_echo = (state.serial.tx_local_echo)();
        let b = bridge.clone();
        let s = state;
        spawn(async move {
            execute_macro_send(cmd, is_hex, ending, port, local_echo, b, s).await;
        });
    };

    let on_add = move |_| {
        editing_id.set(None);
        new_label.set(String::new());
        new_cmd.set(String::new());
        new_hex.set(false);
        new_ending.set(LineEnding::None);
        show_form.set(true);
    };

    rsx! {
        div { class: "flex gap-2 p-2 bg-background-dark border-t border-[#2a2e33] min-h-[40px] items-center overflow-x-auto",
            MacroList { storage, context_menu, on_send, on_add }

            if let Some((id, x, y)) = context_menu() {
                MacroContextMenu {
                    id, x, y,
                    context_menu, show_form, editing_id,
                    new_label, new_cmd, new_hex, new_ending, storage,
                }
            }

            GitHubLink {}

            if show_form() {
                MacroFormModal {
                    show_form, editing_id, new_label, new_cmd, new_hex, new_ending, storage,
                }
            }
        }
    }
}
