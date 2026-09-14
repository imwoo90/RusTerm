//! Serial port status and connection badge component.
//!
//! Visually displays the current serial device link status, providing color-coded status badges,
//! hardware chipset labels, multi-port disambiguation, and user device alias management.

use crate::state::AppState;
use crate::utils::SerialDeviceInfo;
use dioxus::prelude::*;

#[component]
fn ModalHeader(onclose: EventHandler<()>) -> Element {
    rsx! {
        div { class: "flex items-center justify-between border-b border-[#2a2e33] pb-3",
            div { class: "flex items-center gap-2",
                span { class: "material-symbols-outlined text-emerald-500 text-[20px]", "edit_note" }
                h3 { class: "text-sm font-bold text-white tracking-wide", "Set Device Alias" }
            }
            button {
                class: "text-gray-400 hover:text-white transition-colors text-lg leading-none",
                onclick: move |_| onclose.call(()),
                "✕"
            }
        }
    }
}

#[component]
fn ModalBody(
    description: String,
    alias_text: Signal<String>,
    onenter: EventHandler<()>,
    onescape: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "flex flex-col gap-1.5",
            p { class: "text-xs text-gray-400",
                "Assign a custom alias to easily distinguish between multiple ports."
            }
            p { class: "text-[11px] text-gray-500 font-mono", "{description}" }
        }
        input {
            class: "w-full px-3 py-2 bg-[#0d0e10] border border-[#2a2e33] rounded-lg text-xs text-white placeholder-gray-500 focus:outline-none focus:border-emerald-500 transition-colors",
            r#type: "text",
            placeholder: "e.g. COM3, Sensor Board, Motor Controller",
            value: "{alias_text}",
            oninput: move |e| alias_text.set(e.value()),
            onkeydown: move |e| {
                if e.key() == Key::Enter {
                    onenter.call(());
                } else if e.key() == Key::Escape {
                    onescape.call(());
                }
            },
        }
    }
}

#[component]
fn ModalFooter(
    onreset: EventHandler<()>,
    onclose: EventHandler<()>,
    onsave: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "flex items-center justify-between pt-2 border-t border-[#2a2e33]",
            button {
                class: "px-3 py-1.5 text-xs text-red-400 hover:text-red-300 hover:bg-red-500/10 rounded-lg transition-colors",
                onclick: move |_| onreset.call(()),
                "Reset"
            }
            div { class: "flex items-center gap-2",
                button {
                    class: "px-3 py-1.5 text-xs text-gray-400 hover:text-white transition-colors",
                    onclick: move |_| onclose.call(()),
                    "Cancel"
                }
                button {
                    class: "px-3 py-1.5 text-xs font-bold bg-emerald-500 text-black hover:bg-emerald-400 rounded-lg transition-colors shadow-md",
                    onclick: move |_| onsave.call(()),
                    "Save"
                }
            }
        }
    }
}

#[component]
fn AliasModal(
    device_info: SerialDeviceInfo,
    onclose: EventHandler<()>,
    onsave: EventHandler<String>,
    onreset: EventHandler<()>,
) -> Element {
    let alias_text = use_signal(|| device_info.alias.clone().unwrap_or_default());

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/20 backdrop-blur-[1px] animate-fadeIn",
            onclick: move |_| onclose.call(()),
            div {
                class: "w-full max-w-sm bg-[#16181a] border border-[#2a2e33] rounded-xl p-5 shadow-2xl flex flex-col gap-4 animate-scaleUp",
                onclick: move |e| e.stop_propagation(),

                ModalHeader { onclose }
                ModalBody {
                    description: device_info.description.clone(),
                    alias_text,
                    onenter: move |_| onsave.call(alias_text()),
                    onescape: move |_| onclose.call(()),
                }
                ModalFooter {
                    onreset,
                    onclose,
                    onsave: move |_| onsave.call(alias_text()),
                }
            }
        }
    }
}

#[component]
fn DisconnectedBadge() -> Element {
    rsx! {
        div {
            class: "flex items-center gap-2 px-3 py-1.5 bg-[#16181a] rounded-lg border border-[#2a2e33] h-9",
            title: "No serial device connected",
            span { class: "material-symbols-outlined text-gray-500 text-[18px]", "usb_off" }
            span { class: "text-xs font-bold text-gray-500 font-mono", "No Device" }
        }
    }
}

#[component]
fn ConnectedBadge(info: SerialDeviceInfo, onclick: EventHandler<()>) -> Element {
    let is_sim = info.label == "Simulation";
    let border = if is_sim {
        "border-yellow-500/30"
    } else {
        "border-emerald-500/30 hover:border-emerald-500/60 cursor-pointer group"
    };
    let icon = if is_sim { "bug_report" } else { "usb" };
    let icon_color = if is_sim { "text-yellow-500" } else { "text-emerald-500" };
    let text_color = if is_sim { "text-yellow-400" } else { "text-emerald-400" };
    let tooltip = if is_sim {
        info.description.clone()
    } else {
        format!("{} (Click to edit alias)", info.description)
    };

    rsx! {
        div {
            class: "flex items-center gap-2 px-3 py-1.5 bg-[#16181a] rounded-lg border {border} h-9 transition-colors shadow-sm",
            title: "{tooltip}",
            onclick: move |_| {
                if !is_sim {
                    onclick.call(());
                }
            },
            span { class: "material-symbols-outlined {icon_color} text-[18px]", "{icon}" }
            span { class: "text-xs font-bold {text_color} font-mono tracking-tight", "{info.label}" }
            if !is_sim {
                span {
                    class: "material-symbols-outlined text-[13px] text-gray-500 group-hover:text-emerald-400 transition-colors ml-0.5 opacity-60 group-hover:opacity-100",
                    "edit"
                }
            }
        }
    }
}

#[component]
pub fn PortStatus(device_info: Option<SerialDeviceInfo>) -> Element {
    let state = use_context::<AppState>();
    let mut show_alias_modal = use_signal(|| false);

    let Some(info) = device_info else {
        return rsx! { DisconnectedBadge {} };
    };

    let info_modal = info.clone();
    let info_save = info.clone();
    let info_reset = info.clone();

    rsx! {
        ConnectedBadge {
            info: info.clone(),
            onclick: move |_| show_alias_modal.set(true),
        }

        if show_alias_modal() {
            AliasModal {
                device_info: info_modal,
                onclose: move |_| show_alias_modal.set(false),
                onsave: move |alias: String| {
                    crate::utils::set_stored_alias(
                        info_save.vid,
                        info_save.pid,
                        info_save.port_index,
                        &alias,
                    );
                    let mut updated = info_save.clone();
                    updated.update_alias(Some(alias));
                    { state.conn.device_info }.set(Some(updated));
                    state.success("Device alias saved");
                    show_alias_modal.set(false);
                },
                onreset: move |_| {
                    crate::utils::set_stored_alias(
                        info_reset.vid,
                        info_reset.pid,
                        info_reset.port_index,
                        "",
                    );
                    let mut updated = info_reset.clone();
                    updated.update_alias(None);
                    { state.conn.device_info }.set(Some(updated));
                    state.info("Device alias reset");
                    show_alias_modal.set(false);
                },
            }
        }
    }
}
