use crate::components::monitor::utils::style::get_highlight_classes;
use crate::components::ui::{CustomSelect, IconButton, PanelHeader};
use crate::config::HIGHLIGHT_COLORS;
use crate::state::{AppState, LineEnding};
use crate::utils::serial;
use crate::utils::{format_hex_input, parse_hex_string, CommandHistory};
use dioxus::prelude::*;
use futures_util::StreamExt;
use js_sys::Uint8Array;

#[component]
pub fn TransmitBar() -> Element {
    let mut state = use_context::<AppState>();
    let mut input_value = use_signal(String::new);
    let mut history = use_signal(CommandHistory::load);
    let mut history_index = use_signal(|| None::<usize>);
    let mut is_hex_input = use_signal(|| false);

    // Highlight Panel Signals
    let mut index_open = use_signal(|| false);
    let show_highlights = (state.ui.show_highlights)();

    // Line Ending Dropdown Signal
    let mut show_line_ending_menu = use_signal(|| false);

    let bridge = crate::hooks::use_worker_controller();

    let send_task = use_coroutine(move |mut rx| async move {
        while rx.next().await.is_some() {
            let text = input_value();
            let ending = *state.serial.tx_line_ending.peek();

            if text.is_empty() && matches!(ending, LineEnding::None) {
                continue;
            }

            let is_hex = is_hex_input();
            let local_echo = *state.serial.tx_local_echo.peek();
            let port = state.conn.port.peek().as_ref().cloned();

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
                            continue;
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
    });

    rsx! {
        div { class: "flex-1 relative flex gap-2 min-w-0",
            div { class: "relative flex-1 group",
                input {
                    class: "w-full h-full bg-[#0d0f10] text-sm text-white placeholder-gray-600 px-4 pr-32 rounded-lg border border-[#2a2e33] focus:border-primary/50 focus:shadow-glow outline-none shadow-inset-input transition-all font-mono",
                    placeholder: "Send command...",
                    "type": "text",
                    value: "{input_value}",
                    id: "transmit-input",
                    oninput: move |evt| {
                        if is_hex_input() {
                            input_value.set(format_hex_input(&evt.value()));
                        } else {
                            input_value.set(evt.value());
                        }
                    },
                    onkeydown: move |evt| {
                        match evt.key() {
                            Key::Enter => send_task.send(()),
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
                                evt.prevent_default();
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
                                evt.prevent_default();
                            }
                            _ => {}
                        }
                    },
                }
                div { class: "absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-2",
                    // Local Echo Toggle
                    div { class: "relative group/echo",
                        button {
                            class: "w-8 h-8 rounded flex items-center justify-center transition-colors pb-1",
                            class: if (state.serial.tx_local_echo)() { "text-primary bg-primary/10 hover:bg-primary/20" } else { "text-gray-500 hover:text-gray-300 hover:bg-white/5" },
                            onclick: move |_| state.serial.tx_local_echo.set(!(state.serial.tx_local_echo)()),
                            title: "Local Echo",
                            span { class: "material-symbols-outlined text-[20px]", "visibility" }
                        }
                        div { class: "absolute -bottom-1.5 right-0 left-0 flex justify-center pointer-events-none",
                            span {
                                class: "text-[9px] font-bold tracking-wider scale-75 origin-top transition-colors",
                                class: if (state.serial.tx_local_echo)() { "text-primary" } else { "text-gray-500" },
                                "ECHO"
                            }
                        }
                    }

                    // Hex Input Toggle
                    div { class: "relative group/hex",
                        button {
                            class: "w-8 h-8 rounded flex items-center justify-center transition-colors pb-1",
                            class: if is_hex_input() { "text-primary bg-primary/10 hover:bg-primary/20" } else { "text-gray-500 hover:text-gray-300 hover:bg-white/5" },
                            onclick: move |_| is_hex_input.set(!is_hex_input()),
                            title: "HEX Input",
                            span { class: "material-symbols-outlined text-[20px]", "hexagon" }
                        }
                        div { class: "absolute -bottom-1.5 right-0 left-0 flex justify-center pointer-events-none",
                            span {
                                class: "text-[9px] font-bold tracking-wider scale-75 origin-top transition-colors",
                                class: if is_hex_input() { "text-primary" } else { "text-gray-500" },
                                "HEX"
                            }
                        }
                    }

                    // Line Ending Dropdown
                    div { class: "relative group/le",
                        button {
                            class: "w-8 h-8 rounded flex items-center justify-center transition-colors pb-1",
                            class: if show_line_ending_menu() {
                                "text-gray-300 bg-white/10"
                            } else if (state.serial.tx_line_ending)() != LineEnding::None {
                                "text-primary bg-primary/10 hover:bg-primary/20"
                            } else {
                                "text-gray-500 hover:text-gray-300 hover:bg-white/5"
                            },
                            onclick: move |_| show_line_ending_menu.set(!show_line_ending_menu()),
                            title: "Line Ending",
                            span { class: "material-symbols-outlined text-[20px]", "keyboard_return" }
                        }
                        div { class: "absolute -bottom-1.5 right-0 left-0 flex justify-center pointer-events-none",
                             span {
                                class: "text-[9px] font-bold scale-75 origin-top transition-colors",
                                class: if (state.serial.tx_line_ending)() != LineEnding::None { "text-primary" } else { "text-gray-500" },
                                match (state.serial.tx_line_ending)() {
                                    LineEnding::None => "NONE",
                                    LineEnding::NL => "LF",
                                    LineEnding::CR => "CR",
                                    LineEnding::NLCR => "CRLF",
                                }
                             }
                        }

                        if show_line_ending_menu() {
                            // Backdrop to close when clicking outside
                            div {
                                class: "fixed inset-0 z-40 cursor-default",
                                onclick: move |_| show_line_ending_menu.set(false),
                            }

                            div {
                                class: "absolute top-full right-0 mt-2 w-24 bg-[#16181a] border border-[#2a2e33] rounded-lg shadow-xl overflow-hidden z-50 flex flex-col py-1",
                                for ending in [LineEnding::None, LineEnding::NL, LineEnding::CR, LineEnding::NLCR] {
                                    button {
                                        class: "px-3 py-2 text-[11px] font-mono text-left hover:bg-white/5 transition-colors flex items-center justify-between group/item",
                                        onclick: move |_| {
                                            state.serial.tx_line_ending.set(ending);
                                            show_line_ending_menu.set(false);
                                        },
                                        span {
                                            class: if (state.serial.tx_line_ending)() == ending { "text-primary font-bold" } else { "text-gray-400 group-hover/item:text-gray-300" },
                                            match ending {
                                                LineEnding::None => "NONE",
                                                LineEnding::NL => "LF",
                                                LineEnding::CR => "CR",
                                                LineEnding::NLCR => "CRLF",
                                            }
                                        }
                                        if (state.serial.tx_line_ending)() == ending {
                                            span { class: "material-symbols-outlined text-[12px] text-primary", "check" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Highlight Button (Next to Send)
            div { class: "relative h-full aspect-square",
                button {
                    class: "h-full w-full rounded-lg flex items-center justify-center transition-all bg-[#0d0f10] border border-[#2a2e33] hover:border-gray-500 group/highlight",
                    class: if index_open() || (show_highlights && !(state.log.highlights)().is_empty()) {
                        "border-primary text-primary"
                    } else {
                        "text-gray-400 hover:text-white"
                    },
                    onclick: move |_| {
                        index_open.set(!index_open());
                    },
                    title: "Highlight Rules",
                    span { class: "material-symbols-outlined text-[20px]", "ink_highlighter" }
                }

                // Highlight Panel Dropdown (Using the new component below)
                if index_open() {
                    HighlightPanel {
                        visible: true,
                        onclose: move |_| index_open.set(false),
                    }
                }
            }

            button {
                class: "h-full aspect-square bg-primary text-surface rounded-lg flex items-center justify-center hover:bg-white transition-all hover:shadow-[0_0_15px_rgba(255,255,255,0.4)] active:scale-95 group",
                onclick: move |_| send_task.send(()),
                span { class: "material-symbols-outlined text-[20px] group-hover:rotate-45 transition-transform duration-300",
                    "send"
                }
            }
        }
    }
}

#[component]
fn HighlightPanel(visible: bool, onclose: EventHandler<()>) -> Element {
    let state = use_context::<AppState>();
    let highlights = (state.log.highlights)();

    rsx! {
        div {
            class: "fixed inset-0 z-40 cursor-default",
            onclick: move |_| onclose.call(()),
        }
        div { class: "absolute top-full right-0 mt-2 w-80 z-50 bg-[#16181a] rounded-xl border border-white/10 shadow-2xl p-4 animate-in fade-in zoom-in-95 duration-200 origin-top-right",
            div { class: "flex flex-col gap-3",
                PanelHeader {
                    title: "Active Highlights",
                    subtitle: Some(format!("{} rules", highlights.len())),
                }

                div { class: "flex items-center justify-between",
                    span { class: "text-[10px] uppercase text-gray-500 font-bold", "Enable Highlighting" }
                    crate::components::ui::ToggleSwitch {
                        label: "",
                        active: (state.ui.show_highlights)(),
                        onclick: move |_| state.ui.toggle_highlights(),
                    }
                }

                div { class: "flex flex-wrap gap-2 min-h-[40px] p-2 bg-[#0d0f10] rounded border border-[#2a2e33]",
                    if highlights.is_empty() {
                        span { class: "text-xs text-gray-600 italic px-1", "No rules added" }
                    }
                    for h in highlights {
                        HighlightTag {
                            color: h.color,
                            label: h.text.clone(),
                            onremove: move |_| {
                                let state = use_context::<AppState>();
                                state.log.remove_highlight(h.id);
                            },
                        }
                    }
                }
                HighlightInput {}
            }
        }
    }
}

#[component]
fn HighlightTag(color: &'static str, label: String, onremove: EventHandler<MouseEvent>) -> Element {
    let (border_class, text_class) = get_highlight_classes(color);

    rsx! {
        div { class: "flex items-center gap-2 pl-3 pr-2 py-1.5 bg-[#0d0f10] border {border_class} rounded-full group transition-colors",
            span { class: "text-xs font-bold {text_class}", "{label}" }
            IconButton {
                icon: "close",
                icon_class: "text-[14px]",
                class: "ml-1 w-4 h-4 rounded-full",
                onclick: move |evt| onremove.call(evt),
            }
        }
    }
}

#[component]
fn HighlightInput() -> Element {
    let mut new_text = use_signal(String::new);
    let mut add_highlight_logic = move || {
        let text = new_text.read().trim().to_string();
        if !text.is_empty() {
            let state = use_context::<AppState>();
            let list = state.log.highlights.read().clone();

            let used_colors: std::collections::HashSet<&str> =
                list.iter().map(|h| h.color).collect();
            let color = HIGHLIGHT_COLORS
                .iter()
                .find(|&&c| !used_colors.contains(c))
                .copied()
                .unwrap_or_else(|| HIGHLIGHT_COLORS[list.len() % HIGHLIGHT_COLORS.len()]);

            state.log.add_highlight(text, color);
            new_text.set(String::new());
        }
    };

    rsx! {
        div { class: "pt-2 border-t border-white/5 flex gap-2",
            input {
                class: "flex-1 bg-[#0d0f10] text-xs font-medium text-white placeholder-gray-600 px-3 py-2 rounded-lg border border-[#2a2e33] focus:border-primary/50 focus:shadow-glow outline-none transition-all",
                placeholder: "Enter keyword...",
                "type": "text",
                value: "{new_text}",
                oninput: move |evt| new_text.set(evt.value()),
                onkeydown: move |evt| {
                    if evt.key() == Key::Enter {
                        add_highlight_logic();
                    }
                },
            }
            button {
                class: "px-4 rounded-lg bg-primary text-surface font-bold hover:bg-white transition-all active:scale-95 flex items-center gap-2",
                onclick: move |_| add_highlight_logic(),
                span { class: "material-symbols-outlined text-[18px]", "add" }
                span { class: "text-[10px] uppercase tracking-wider", "Add" }
            }
        }
    }
}
