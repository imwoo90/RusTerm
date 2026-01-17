use crate::serial;
use crate::state::{AppState, LineEnding};
use crate::utils::MacroStorage;
use dioxus::prelude::*;

#[component]
pub fn MacroBar() -> Element {
    let mut state = use_context::<AppState>();
    let mut storage = use_signal(|| MacroStorage::load());
    let mut show_form = use_signal(|| false);

    let mut new_label = use_signal(String::new);
    let mut new_cmd = use_signal(String::new);

    let send_cmd = move |cmd: String| {
        spawn(async move {
            let mut data = cmd.into_bytes();
            match (state.line_ending)() {
                LineEnding::NL => data.push(b'\n'),
                LineEnding::CR => data.push(b'\r'),
                LineEnding::NLCR => {
                    data.push(b'\r');
                    data.push(b'\n');
                }
                _ => {}
            }
            if let Some(wrapper) = (state.port)() {
                let _ = serial::send_data(&wrapper.0, &data).await;
            }
        });
    };

    rsx! {
        div { class: "flex flex-wrap gap-2 p-2 bg-background-dark border-t border-[#2a2e33] min-h-[40px] items-center",
            for item in storage.read().get_items() {
                 button {
                    key: "{item.id}",
                    class: "px-3 py-1 bg-[#2a2e33] hover:bg-primary hover:text-white rounded text-xs font-mono transition-colors border border-gray-700 select-none",
                    onclick: move |_| send_cmd(item.command.clone()),
                    oncontextmenu: move |evt| {
                        evt.prevent_default();
                        storage.write().remove(item.id);
                    },
                    title: "Right-click to remove",
                    "{item.label}"
                 }
            }

            // Add Button
            button {
                class: "w-6 h-6 flex items-center justify-center bg-[#1a1c1e] text-gray-400 hover:text-white rounded text-xs border border-dashed border-gray-700 hover:border-gray-500 transition-colors",
                onclick: move |_| show_form.set(!show_form()),
                title: "Add Macro",
                "+"
            }

            // Form Modal
            if show_form() {
                div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm",
                    div { class: "bg-[#16181a] p-4 rounded-xl border border-[#2a2e33] w-80 shadow-2xl",
                        h3 { class: "text-sm font-bold text-gray-300 mb-3", "Add Quick Command" }
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
                                input {
                                    class: "w-full bg-[#0d0f10] text-white p-2 rounded border border-[#2a2e33] text-xs font-mono focus:border-primary/50 outline-none",
                                    placeholder: "e.g. AT+RST",
                                    value: "{new_cmd}",
                                    oninput: move |e| new_cmd.set(e.value())
                                }
                            }
                        }
                        div { class: "flex justify-end gap-2 mt-4",
                            button {
                                class: "px-3 py-1.5 text-xs text-gray-400 hover:text-white transition-colors",
                                onclick: move |_| show_form.set(false),
                                "Cancel"
                            }
                            button {
                                class: "px-3 py-1.5 text-xs bg-primary text-white rounded hover:bg-primary-hover shadow-lg shadow-primary/20 transition-all active:scale-95",
                                onclick: move |_| {
                                    if !new_label().is_empty() && !new_cmd().is_empty() {
                                        storage.write().add(new_label(), new_cmd());
                                        new_label.set(String::new());
                                        new_cmd.set(String::new());
                                        show_form.set(false);
                                    }
                                },
                                "Add"
                            }
                        }
                    }
                    // Click outside to close
                    div { class: "absolute inset-0 -z-10", onclick: move |_| show_form.set(false) }
                }
            }
        }
    }
}
