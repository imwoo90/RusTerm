use crate::components::common::{IconButton, LineEndSelector, PanelHeader};
use crate::state::{AppState, Highlight, HIGHLIGHT_COLORS};
use dioxus::prelude::*;

#[component]
pub fn FilterBar() -> Element {
    let mut state = use_context::<AppState>();
    let show_highlights = (state.show_highlights)();
    let mut index_open = use_signal(|| false);

    // RX Settings
    let rx_ending = (state.rx_line_ending)();

    rsx! {
        div { class: "shrink-0 p-2 z-10 border-b border-[#2a2e33] bg-[#0d0f10]",
            div { class: "flex gap-3 items-center w-full min-w-[600px]",

                // --- Left: View Settings & RX (Aligns with Filter Input: flex-[1.3]) ---
                div { class: "flex-[1.3] flex items-center gap-4 min-w-0 pl-1",
                    div { class: "flex items-center gap-2",
                        // Timestamp Button
                        button {
                            class: "px-2 py-1 rounded text-[10px] font-bold border transition-colors select-none",
                            class: if (state.show_timestamps)() { "bg-primary/20 text-primary border-primary/30" } else { "text-gray-500 border-transparent hover:text-gray-300 bg-[#2a2e33]/50" },
                            onclick: move |_| {
                                let v = (state.show_timestamps)();
                                state.show_timestamps.set(!v);
                            },
                            "TIME"
                        }

                        // Hex View Button
                        button {
                            class: "px-2 py-1 rounded text-[10px] font-bold border transition-colors select-none",
                            class: if (state.is_hex_view)() { "bg-primary/20 text-primary border-primary/30" } else { "text-gray-500 border-transparent hover:text-gray-300 bg-[#2a2e33]/50" },
                            onclick: move |_| state.is_hex_view.set(!(state.is_hex_view)()),
                            "HEX"
                        }
                    }

                    div { class: "w-px h-4 bg-[#2a2e33]" }

                    // RX Line Ending
                    LineEndSelector {
                        label: "RX PARSE",
                        selected: rx_ending,
                        onselect: move |val| state.rx_line_ending.set(val),
                        active_class: "bg-emerald-500/20 text-emerald-500 border-emerald-500/20",
                        is_rx: true,
                    }
                }

                // --- Divider (Matches InputBar Divider Position) ---
                div { class: "w-px h-6 bg-[#2a2e33]" }

                // --- Right: TX Settings & Highlight (Aligns with Send Input: flex-1) ---
                div { class: "flex-1 flex items-center justify-between min-w-0 pr-1",
                    // TX Line Ending
                    LineEndSelector {
                        label: "TX APPEND",
                        selected: (state.line_ending)(),
                        onselect: move |val| state.line_ending.set(val),
                        active_class: "bg-primary/20 text-primary border-primary/20",
                        is_rx: false,
                    }

                    // Highlight Panel Toggle
                    div { class: "relative ml-4",
                        IconButton {
                            icon: "ink_highlighter",
                            active: index_open(),
                            class: "w-8 h-8 rounded-lg border border-[#2a2e33] bg-[#0d0f10] hover:border-gray-500",
                            icon_class: "text-[18px]",
                            onclick: move |_| {
                                let cur = index_open();
                                index_open.set(!cur);
                                if !cur && !show_highlights {
                                    state.show_highlights.set(true);
                                }
                            },
                            title: "Highlight Rules",
                        }

                        // Highlight Panel Dropdown
                        if index_open() {
                            HighlightPanel {
                                visible: true,
                                onclose: move |_| index_open.set(false),
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn HighlightPanel(visible: bool, onclose: EventHandler<()>) -> Element {
    let mut state = use_context::<AppState>();
    let highlights = (state.highlights)();

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
                    crate::components::common::ToggleSwitch {
                        label: "",
                        active: (state.show_highlights)(),
                        onclick: move |_| state.show_highlights.set(!(state.show_highlights)()),
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
                                let mut state = use_context::<AppState>();
                                let mut list = state.highlights.read().clone();
                                list.retain(|item| item.id != h.id);
                                state.highlights.set(list);
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
    let (border_class, text_class) = match color {
        "red" => ("border-red-500/30 hover:border-red-500/60", "text-red-400"),
        "blue" => (
            "border-blue-500/30 hover:border-blue-500/60",
            "text-blue-400",
        ),
        "yellow" => (
            "border-yellow-500/30 hover:border-yellow-500/60",
            "text-yellow-400",
        ),
        "green" => (
            "border-green-500/30 hover:border-green-500/60",
            "text-green-400",
        ),
        "purple" => (
            "border-purple-500/30 hover:border-purple-500/60",
            "text-purple-400",
        ),
        "orange" => (
            "border-orange-500/30 hover:border-orange-500/60",
            "text-orange-400",
        ),
        "teal" => (
            "border-teal-500/30 hover:border-teal-500/60",
            "text-teal-400",
        ),
        "pink" => (
            "border-pink-500/30 hover:border-pink-500/60",
            "text-pink-400",
        ),
        "indigo" => (
            "border-indigo-500/30 hover:border-indigo-500/60",
            "text-indigo-400",
        ),
        "lime" => (
            "border-lime-500/30 hover:border-lime-500/60",
            "text-lime-400",
        ),
        "cyan" => (
            "border-cyan-500/30 hover:border-cyan-500/60",
            "text-cyan-400",
        ),
        "rose" => (
            "border-rose-500/30 hover:border-rose-500/60",
            "text-rose-400",
        ),
        "fuchsia" => (
            "border-fuchsia-500/30 hover:border-fuchsia-500/60",
            "text-fuchsia-400",
        ),
        "amber" => (
            "border-amber-500/30 hover:border-amber-500/60",
            "text-amber-400",
        ),
        "emerald" => (
            "border-emerald-500/30 hover:border-emerald-500/60",
            "text-emerald-400",
        ),
        "sky" => ("border-sky-500/30 hover:border-sky-500/60", "text-sky-400"),
        "violet" => (
            "border-violet-500/30 hover:border-violet-500/60",
            "text-violet-400",
        ),
        _ => ("border-primary/30 hover:border-primary/60", "text-primary"),
    };

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
    let mut new_text = use_signal(|| String::new());

    let mut add_highlight_logic = move || {
        let text = new_text.read().trim().to_string();
        if !text.is_empty() {
            let mut state = use_context::<AppState>();
            let mut list = state.highlights.read().clone();

            let used_colors: std::collections::HashSet<&str> =
                list.iter().map(|h| h.color).collect();
            let color = HIGHLIGHT_COLORS
                .iter()
                .find(|&&c| !used_colors.contains(c))
                .copied()
                .unwrap_or_else(|| HIGHLIGHT_COLORS[list.len() % HIGHLIGHT_COLORS.len()]);

            let next_id = list.iter().map(|h| h.id).max().unwrap_or(0) + 1;
            list.push(Highlight {
                id: next_id,
                text,
                color,
            });
            state.highlights.set(list);
            new_text.set(String::new());
        }
    };

    rsx! {
        div { class: "pt-2 border-t border-white/5 flex gap-2",
            input {
                class: "flex-1 bg-[#0d0f10] text-xs font-medium text-white placeholder-gray-600 px-3 py-2 rounded-lg border border-[#2a2e33] focus:border-primary/50 focus:shadow-glow outline-none transition-all",
                placeholder: "Enter keyword to highlight...",
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
