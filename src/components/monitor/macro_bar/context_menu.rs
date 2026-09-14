//! Context menu component for the quick command macro bar.
//!
//! Provides edit and delete actions triggered by right-clicking on macro buttons.
//! It automatically positions itself relative to cursor coordinates and handles click-outside dismissal.

use crate::state::LineEnding;
use crate::utils::MacroStorage;
use dioxus::prelude::*;

#[component]
fn MenuItem(
    onclick: EventHandler<()>,
    class: &'static str,
    icon: &'static str,
    label: &'static str,
) -> Element {
    rsx! {
        button {
            class: "w-full text-left px-3 py-1.5 text-xs transition-colors flex items-center gap-2 {class}",
            onclick: move |_| onclick.call(()),
            span { class: "material-symbols-outlined text-[14px]", "{icon}" }
            "{label}"
        }
    }
}

#[component]
pub fn MacroContextMenu(
    id: u64,
    x: i32,
    y: i32,
    mut context_menu: Signal<Option<(u64, i32, i32)>>,
    mut show_form: Signal<bool>,
    mut editing_id: Signal<Option<u64>>,
    mut new_label: Signal<String>,
    mut new_cmd: Signal<String>,
    mut new_hex: Signal<bool>,
    mut new_ending: Signal<LineEnding>,
    mut storage: Signal<MacroStorage>,
) -> Element {
    let on_edit = move |_| {
        context_menu.set(None);
        if let Some(item) = storage.read().get(id) {
            editing_id.set(Some(id));
            new_label.set(item.label);
            new_cmd.set(item.command);
            new_hex.set(item.is_hex);
            new_ending.set(item.line_ending);
            show_form.set(true);
        }
    };

    let on_delete = move |_| {
        storage.write().remove(id);
        context_menu.set(None);
    };

    rsx! {
        div {
            class: "fixed inset-0 z-50",
            onclick: move |_| context_menu.set(None),
            div {
                class: "absolute bg-[#16181a] border border-[#2a2e33] rounded-lg shadow-xl py-1 z-50 min-w-[120px]",
                style: "top: {y}px; left: {x}px; transform: translateY(-100%);",
                onclick: |evt| evt.stop_propagation(),

                MenuItem {
                    onclick: on_edit,
                    class: "text-gray-300 hover:bg-white/5 hover:text-white",
                    icon: "edit",
                    label: "Edit",
                }
                MenuItem {
                    onclick: on_delete,
                    class: "text-red-400 hover:bg-red-900/20 hover:text-red-300",
                    icon: "delete",
                    label: "Delete",
                }
            }
        }
    }
}
