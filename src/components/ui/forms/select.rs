//! # Select Dropdown Component
//!
//! ## Overview
//! Provides a styled generic select dropdown component supporting keyboard navigation, custom option renderers, and accessible focus states.
//!
//! ## Search Tags
//! #select, #dropdown, #combo-box, #form-control

use dioxus::prelude::*;

#[component]
fn SelectChevron(is_open: bool) -> Element {
    rsx! {
        span {
            class: "material-symbols-outlined text-[18px] text-gray-500 group-hover/select:text-primary transition-all duration-300",
            class: if is_open { "rotate-180 text-primary" } else { "" },
            "expand_more"
        }
    }
}

#[component]
fn SelectDropdownMenu(
    options: Vec<&'static str>,
    selected: String,
    onselect: EventHandler<String>,
    onclose: EventHandler<()>,
    scrollable: bool,
) -> Element {
    rsx! {
        div {
            class: "fixed inset-0 cursor-default",
            onclick: move |_| onclose.call(()),
        }
        div {
            class: "absolute top-full left-0 right-0 mt-1 z-50 bg-[#16181a] border border-white/10 rounded-xl shadow-2xl py-1 overflow-hidden animate-in fade-in slide-in-from-top-2 duration-200",
            class: if scrollable { "max-h-60 overflow-y-auto custom-scrollbar" } else { "" },
            onclick: move |e| e.stop_propagation(),
            for opt in options {
                button {
                    class: "w-full text-left px-3 py-2 text-[11px] font-bold transition-all duration-150",
                    class: if opt == selected { "bg-primary/20 text-primary" } else { "text-gray-400 hover:bg-white/5 hover:text-white" },
                    onclick: move |_| onselect.call(opt.to_string()),
                    "{opt}"
                }
            }
        }
    }
}

/// A reusable custom select component with premium styling.
#[component]
pub fn CustomSelect(
    options: Vec<&'static str>,
    selected: String,
    onchange: EventHandler<String>,
    #[props(default = "w-full")] class: &'static str,
    #[props(default = false)] disabled: bool,
) -> Element {
    let mut is_open = use_signal(|| false);

    let onselect = move |opt: String| {
        onchange.call(opt);
        is_open.set(false);
    };

    rsx! {
        div {
            class: "relative {class} group/select",
            class: if is_open() { "z-50" },
            button {
                class: if disabled { "w-full flex items-center justify-between bg-[#0d0f10] border border-[#2a2e33] rounded-lg text-xs font-bold text-gray-500 py-2 px-3 opacity-50 cursor-not-allowed" } else { "w-full flex items-center justify-between bg-[#0d0f10] border border-[#2a2e33] rounded-lg text-xs font-bold text-gray-300 py-2 px-3 hover:bg-[#16181a] hover:border-primary/50 transition-all duration-200 outline-none focus:border-primary/50" },
                disabled: "{disabled}",
                onclick: move |e| {
                    if !disabled {
                        e.stop_propagation();
                        is_open.set(!is_open());
                    }
                },
                span { "{selected}" }
                SelectChevron { is_open: is_open() }
            }

            if is_open() {
                SelectDropdownMenu {
                    options,
                    selected: selected.clone(),
                    onselect,
                    onclose: move |_| is_open.set(false),
                    scrollable: false,
                }
            }
        }
    }
}

#[component]
fn ComboInputControls(
    selected: String,
    disabled: bool,
    is_open: Signal<bool>,
    oninput: EventHandler<String>,
    ontoggle: EventHandler<MouseEvent>,
    onfocus: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: if disabled { "w-full flex items-center justify-between bg-[#0d0f10] border border-[#2a2e33] rounded-lg py-0 px-0 opacity-50 cursor-not-allowed" } else { "w-full flex items-center justify-between bg-[#0d0f10] border border-[#2a2e33] rounded-lg py-0 px-0 hover:bg-[#16181a] hover:border-primary/50 transition-all duration-200 focus-within:border-primary/50" },
            input {
                class: if disabled { "w-full bg-transparent border-none text-xs font-bold text-gray-300 py-2 pl-3 outline-none placeholder-gray-600 cursor-not-allowed" } else { "w-full bg-transparent border-none text-xs font-bold text-gray-300 py-2 pl-3 outline-none placeholder-gray-600" },
                value: "{selected}",
                disabled: "{disabled}",
                oninput: move |evt: FormEvent| oninput.call(evt.value()),
                onfocus: move |_| onfocus.call(()),
            }

            button {
                class: if disabled { "flex items-center justify-center px-2 py-2 outline-none border-l border-[#2a2e33] cursor-not-allowed" } else { "flex items-center justify-center px-2 py-2 outline-none border-l border-[#2a2e33] cursor-pointer" },
                disabled: "{disabled}",
                onclick: move |e| ontoggle.call(e),
                SelectChevron { is_open: is_open() }
            }
        }
    }
}

/// A select component that also allows custom user input
#[component]
pub fn CustomInputSelect(
    options: Vec<&'static str>,
    selected: String,
    onchange: EventHandler<String>,
    #[props(default = "w-full")] class: &'static str,
    #[props(default = false)] disabled: bool,
) -> Element {
    let mut is_open = use_signal(|| false);

    let onselect = move |opt: String| {
        onchange.call(opt);
        is_open.set(false);
    };

    rsx! {
        div {
            class: "relative {class} group/select",
            class: if is_open() { "z-50" },
            ComboInputControls {
                selected: selected.clone(),
                disabled,
                is_open,
                oninput: move |val| onchange.call(val),
                ontoggle: move |e: MouseEvent| {
                    if !disabled {
                        e.stop_propagation();
                        is_open.set(!is_open());
                    }
                },
                onfocus: move |_| is_open.set(false),
            }

            if is_open() {
                SelectDropdownMenu {
                    options,
                    selected: selected.clone(),
                    onselect,
                    onclose: move |_| is_open.set(false),
                    scrollable: true,
                }
            }
        }
    }
}
