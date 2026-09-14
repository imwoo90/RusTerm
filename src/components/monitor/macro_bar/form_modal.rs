//! Modal dialog component for creating and updating quick command macros.
//!
//! Provides inputs for label, command string, HEX toggle, and line ending selection.
//! Validates hexadecimal input before saving and handles local state cleanup on submit or cancellation.

use crate::components::ui::forms::CommandInputGroup;
use crate::state::{AppState, LineEnding};
use crate::utils::{parse_hex_string, MacroStorage};
use dioxus::prelude::*;

fn save_macro_item(
    editing_id: Option<u64>,
    label: String,
    cmd: String,
    is_hex: bool,
    ending: LineEnding,
    storage: &mut Signal<MacroStorage>,
    state: &AppState,
) -> bool {
    if label.is_empty() || cmd.is_empty() {
        state.error("Please fill in all fields");
        return false;
    }
    if is_hex {
        if let Err(e) = parse_hex_string(&cmd) {
            state.error(&format!("Macro Hex Error: {}", e));
            return false;
        }
    }
    if let Some(id) = editing_id {
        storage.write().update(id, label, cmd, is_hex, ending);
        state.success("Macro Updated");
    } else {
        storage.write().add(label, cmd, is_hex, ending);
        state.success("Macro Added");
    }
    true
}

#[component]
fn MacroFormInputs(
    new_label: Signal<String>,
    new_cmd: Signal<String>,
    new_hex: Signal<bool>,
    new_ending: Signal<LineEnding>,
) -> Element {
    rsx! {
        div { class: "space-y-3",
            div {
                label { class: "block text-[10px] uppercase text-gray-500 font-bold mb-1", "Label" }
                input {
                    class: "w-full bg-[#0d0f10] text-white p-2 rounded border border-[#2a2e33] text-xs focus:border-primary/50 outline-none",
                    placeholder: "e.g. Reboot",
                    value: "{new_label}",
                    oninput: move |e| new_label.set(e.value()),
                    autofocus: true,
                }
            }
            div {
                label { class: "block text-[10px] uppercase text-gray-500 font-bold mb-1", "Command" }
                CommandInputGroup {
                    value: new_cmd,
                    is_hex: new_hex,
                    line_ending: new_ending,
                    echo: None,
                    placeholder: "e.g. AT+RST",
                    on_submit: None,
                    id: None,
                }
            }
        }
    }
}

#[component]
fn MacroModalActions(
    editing_id: Option<u64>,
    on_cancel: EventHandler<()>,
    on_delete: EventHandler<()>,
    on_save: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "flex justify-end gap-2 mt-4",
            button {
                class: "px-3 py-1.5 text-xs text-gray-400 hover:text-white transition-colors",
                onclick: move |_| on_cancel.call(()),
                "Cancel"
            }
            if editing_id.is_some() {
                button {
                    class: "mr-auto px-3 py-1.5 text-xs text-red-400 hover:text-red-300 hover:bg-red-900/20 rounded transition-colors",
                    onclick: move |_| on_delete.call(()),
                    "Delete"
                }
            }
            button {
                class: "px-3 py-1.5 text-xs bg-primary text-white rounded hover:bg-primary-hover shadow-lg shadow-primary/20 transition-all active:scale-95",
                onclick: move |_| on_save.call(()),
                if editing_id.is_some() { "Save" } else { "Add" }
            }
        }
    }
}

#[component]
pub fn MacroFormModal(
    mut show_form: Signal<bool>,
    editing_id: Signal<Option<u64>>,
    mut new_label: Signal<String>,
    new_cmd: Signal<String>,
    new_hex: Signal<bool>,
    new_ending: Signal<LineEnding>,
    mut storage: Signal<MacroStorage>,
) -> Element {
    let state = use_context::<AppState>();

    let on_delete = move |_| {
        if let Some(id) = editing_id() {
            storage.write().remove(id);
            show_form.set(false);
            state.success("Macro Deleted");
        }
    };

    let on_save = move |_| {
        if save_macro_item(
            editing_id(),
            new_label(),
            new_cmd(),
            new_hex(),
            new_ending(),
            &mut storage,
            &state,
        ) {
            new_label.set(String::new());
            show_form.set(false);
        }
    };

    rsx! {
        div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm",
            div { class: "bg-[#16181a] p-4 rounded-xl border border-[#2a2e33] w-80 shadow-2xl",
                h3 { class: "text-sm font-bold text-gray-300 mb-3",
                    if editing_id().is_some() { "Edit Macro" } else { "Add Quick Command" }
                }
                MacroFormInputs { new_label, new_cmd, new_hex, new_ending }
                MacroModalActions {
                    editing_id: editing_id(),
                    on_cancel: move |_| show_form.set(false),
                    on_delete,
                    on_save,
                }
            }
            div {
                class: "absolute inset-0 -z-10",
                onclick: move |_| show_form.set(false),
            }
        }
    }
}
