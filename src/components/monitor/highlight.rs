use crate::components::monitor::utils::style::get_highlight_classes;
use crate::components::ui::{IconButton, PanelHeader};
use crate::config::HIGHLIGHT_COLORS;
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn HighlightButton(is_open: Signal<bool>) -> Element {
    let state = use_context::<AppState>();
    let show_highlights = (state.ui.show_highlights)();

    rsx! {
        if is_open() {
            div {
                class: "fixed inset-0 z-40 cursor-default",
                onclick: move |_| is_open.set(false),
            }
        }
        div {
            class: "relative h-full aspect-square",
            class: if is_open() { "z-50" },
            button {
                class: "h-full w-full rounded-lg flex items-center justify-center transition-all bg-[#0d0f10] border border-[#2a2e33] hover:border-gray-500 group/highlight",
                class: if is_open() || (show_highlights && !(state.log.highlights)().is_empty()) { "border-primary text-primary" } else { "text-gray-400 hover:text-white" },
                onclick: move |_| is_open.set(!is_open()),
                title: "Highlight Rules",
                span { class: "material-symbols-outlined text-[20px]", "ink_highlighter" }
            }

            if is_open() {
                HighlightPanel { visible: true, onclose: move |_| is_open.set(false) }
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
            class: "absolute top-full left-0 mt-2 w-80 z-50 bg-[#16181a] rounded-xl border border-white/10 shadow-2xl p-4 animate-in fade-in zoom-in-95 duration-200 origin-top-left",
            onclick: |evt| evt.stop_propagation(),
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
