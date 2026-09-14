//! # Command Input Bar
//!
//! ## Overview
//! Provides a text entry bar for typing and transmitting serial commands, supporting command history recall via arrow keys and format toggles.
//!
//! ## Search Tags
//! #command-input, #text-entry, #history, #serial-transmit

use crate::state::LineEnding;
use crate::utils::{convert_hex_to_text, convert_text_to_hex, format_hex_input};
use dioxus::prelude::*;

#[component]
fn EchoToggle(mut echo_signal: Signal<bool>) -> Element {
    rsx! {
        div { class: "relative group/echo",
            button {
                class: "w-8 h-8 rounded flex items-center justify-center transition-colors pb-1",
                class: if echo_signal() { "text-primary bg-primary/10 hover:bg-primary/20" } else { "text-gray-500 hover:text-gray-300 hover:bg-white/5" },
                onclick: move |_| echo_signal.set(!echo_signal()),
                title: "Local Echo",
                span { class: "material-symbols-outlined text-[20px]", "visibility" }
            }
            div { class: "absolute -bottom-1.5 right-0 left-0 flex justify-center pointer-events-none",
                span {
                    class: "text-[9px] font-bold tracking-wider scale-75 origin-top transition-colors",
                    class: if echo_signal() { "text-primary" } else { "text-gray-500" },
                    "ECHO"
                }
            }
        }
    }
}

#[component]
fn HexToggle(mut is_hex: Signal<bool>, mut value: Signal<String>) -> Element {
    rsx! {
        div { class: "relative group/hex",
            button {
                class: "w-8 h-8 rounded flex items-center justify-center transition-colors pb-1",
                class: if is_hex() { "text-primary bg-primary/10 hover:bg-primary/20" } else { "text-gray-500 hover:text-gray-300 hover:bg-white/5" },
                onclick: move |_| {
                    let next = !is_hex();
                    let cur = value();
                    if next {
                        value.set(convert_text_to_hex(&cur));
                    } else {
                        value.set(convert_hex_to_text(&cur));
                    }
                    is_hex.set(next);
                },
                title: "HEX Input",
                span { class: "material-symbols-outlined text-[20px]", "hexagon" }
            }
            div { class: "absolute -bottom-1.5 right-0 left-0 flex justify-center pointer-events-none",
                span {
                    class: "text-[9px] font-bold tracking-wider scale-75 origin-top transition-colors",
                    class: if is_hex() { "text-primary" } else { "text-gray-500" },
                    "HEX"
                }
            }
        }
    }
}

fn ending_label(ending: LineEnding) -> &'static str {
    match ending {
        LineEnding::None => "NONE",
        LineEnding::NL => "LF",
        LineEnding::CR => "CR",
        LineEnding::NLCR => "CRLF",
    }
}

#[component]
fn LineEndingMenuItems(
    mut line_ending: Signal<LineEnding>,
    mut show_menu: Signal<bool>,
) -> Element {
    rsx! {
        div { class: "absolute top-full right-0 mt-2 w-24 bg-[#16181a] border border-[#2a2e33] rounded-lg shadow-xl overflow-hidden z-50 flex flex-col py-1",
            for ending in [LineEnding::None, LineEnding::NL, LineEnding::CR, LineEnding::NLCR] {
                button {
                    class: "px-3 py-2 text-[11px] font-mono text-left hover:bg-white/5 transition-colors flex items-center justify-between group/item",
                    onclick: move |_| {
                        line_ending.set(ending);
                        show_menu.set(false);
                    },
                    span {
                        class: if line_ending() == ending { "text-primary font-bold" } else { "text-gray-400 group-hover/item:text-gray-300" },
                        "{ending_label(ending)}"
                    }
                    if line_ending() == ending {
                        span { class: "material-symbols-outlined text-[12px] text-primary", "check" }
                    }
                }
            }
        }
    }
}

#[component]
fn LineEndingDropdown(
    line_ending: Signal<LineEnding>,
    mut show_menu: Signal<bool>,
) -> Element {
    rsx! {
        div { class: "relative group/le",
            button {
                class: "w-8 h-8 rounded flex items-center justify-center transition-colors pb-1",
                class: if show_menu() { "text-gray-300 bg-white/10" } else if line_ending() != LineEnding::None { "text-primary bg-primary/10 hover:bg-primary/20" } else { "text-gray-500 hover:text-gray-300 hover:bg-white/5" },
                onclick: move |_| show_menu.set(!show_menu()),
                title: "Line Ending",
                span { class: "material-symbols-outlined text-[20px]", "keyboard_return" }
            }
            div { class: "absolute -bottom-1.5 right-0 left-0 flex justify-center pointer-events-none",
                span {
                    class: "text-[9px] font-bold scale-75 origin-top transition-colors",
                    class: if line_ending() != LineEnding::None { "text-primary" } else { "text-gray-500" },
                    "{ending_label(line_ending())}"
                }
            }

            if show_menu() {
                LineEndingMenuItems { line_ending, show_menu }
            }
        }
    }
}

#[component]
fn CommandTrailingControls(
    echo: Option<Signal<bool>>,
    is_hex: Signal<bool>,
    value: Signal<String>,
    line_ending: Signal<LineEnding>,
    show_line_ending_menu: Signal<bool>,
) -> Element {
    rsx! {
        div { class: "absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-1",
            if let Some(echo_sig) = echo {
                EchoToggle { echo_signal: echo_sig }
            }
            HexToggle { is_hex, value }
            LineEndingDropdown { line_ending, show_menu: show_line_ending_menu }
        }
    }
}

const INPUT_BASE_CLASS: &str = "w-full h-full min-h-[40px] bg-[#0d0f10] text-sm text-white placeholder-gray-600 px-4 rounded-lg border border-[#2a2e33] focus:border-primary/50 focus:shadow-glow outline-none transition-all font-mono";

#[component]
pub fn CommandInputGroup(
    mut value: Signal<String>,
    is_hex: Signal<bool>,
    line_ending: Signal<LineEnding>,
    echo: Option<Signal<bool>>,
    placeholder: &'static str,
    on_submit: Option<EventHandler<()>>,
    on_keydown: Option<EventHandler<KeyboardEvent>>,
    id: Option<&'static str>,
) -> Element {
    let mut show_line_ending_menu = use_signal(|| false);

    rsx! {
        if show_line_ending_menu() {
            div {
                class: "fixed inset-0 z-40 cursor-default",
                onclick: move |_| show_line_ending_menu.set(false),
            }
        }
        div {
            class: "relative group flex-1",
            class: if show_line_ending_menu() { "z-50" },
            input {
                class: "{INPUT_BASE_CLASS}",
                class: if echo.is_some() { "pr-32" } else { "pr-24" },
                placeholder: "{placeholder}",
                "type": "text",
                value: "{value}",
                id: id.unwrap_or_default(),
                oninput: move |evt| {
                    if is_hex() {
                        value.set(format_hex_input(&evt.value()));
                    } else {
                        value.set(evt.value());
                    }
                },
                onkeydown: move |evt| {
                    if evt.key() == Key::Enter {
                        if let Some(handler) = on_submit {
                            handler.call(());
                        }
                    }
                    if let Some(handler) = on_keydown {
                        handler.call(evt);
                    }
                },
            }
            CommandTrailingControls {
                echo,
                is_hex,
                value,
                line_ending,
                show_line_ending_menu,
            }
        }
    }
}
