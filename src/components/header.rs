//! # Application Top Header
//!
//! ## Overview
//! Hosts application branding, the view mode switcher dropdown (Monitor Log Viewer vs xterm.js Terminal), and the primary connection controls toolbar.
//!
//! ## Search Tags
//! #header, #branding, #mode-switcher, #navigation, #ui

use crate::components::connection_control::ConnectionControl;
use crate::config::APP_SUBTITLE;
use crate::state::{AppState, ViewMode};
use dioxus::prelude::*;

#[component]
fn ModeOptionButton(
    icon: &'static str,
    title: &'static str,
    subtitle: &'static str,
    active: bool,
    border_top: bool,
    onclick: EventHandler<()>,
) -> Element {
    let icon_color = if active { "text-primary" } else { "text-gray-500 group-hover:text-gray-300" };
    let text_color = if active { "text-white" } else { "text-gray-300 group-hover:text-white" };
    let border_class = if border_top { "border-t border-[#2a2e33]" } else { "" };

    rsx! {
        button {
            class: "flex items-center gap-3 px-4 py-2.5 text-left hover:bg-[#2a2e33] transition-colors group {border_class}",
            onclick: move |_| onclick.call(()),
            span { class: "material-symbols-outlined text-[20px] {icon_color}", "{icon}" }
            div { class: "flex flex-col",
                span { class: "text-sm font-medium leading-none mb-0.5 {text_color}", "{title}" }
                span { class: "text-[10px] text-gray-500 group-hover:text-gray-400", "{subtitle}" }
            }
        }
    }
}

#[component]
fn ModeDropdownMenu(mut show_menu: Signal<bool>) -> Element {
    let app = use_context::<AppState>();
    let view_mode = (app.ui.view_mode)();

    rsx! {
        div {
            class: "fixed inset-0 z-40 cursor-default",
            onclick: move |_| show_menu.set(false),
        }
        div { class: "absolute top-full left-0 mt-2 w-48 bg-[#1e2024] border border-[#2a2e33] rounded-xl shadow-2xl overflow-hidden z-50 flex flex-col py-1 animation-files-enter",
            ModeOptionButton {
                icon: "list_alt",
                title: "Monitor",
                subtitle: "Log Viewer & Analysis",
                active: view_mode == ViewMode::Monitoring,
                border_top: false,
                onclick: move |_| {
                    app.ui.set_view_mode(ViewMode::Monitoring);
                    show_menu.set(false);
                },
            }
            ModeOptionButton {
                icon: "terminal",
                title: "Terminal",
                subtitle: "Details for v3.0.0",
                active: view_mode == ViewMode::Terminal,
                border_top: true,
                onclick: move |_| {
                    app.ui.set_view_mode(ViewMode::Terminal);
                    show_menu.set(false);
                },
            }
        }
    }
}

#[component]
fn BrandTriggerButton(mut show_menu: Signal<bool>) -> Element {
    let app = use_context::<AppState>();
    let is_terminal = (app.ui.view_mode)() == ViewMode::Terminal;

    let icon_bg = if is_terminal {
        "bg-gray-700 shadow-gray-900/20"
    } else {
        "bg-linear-to-br from-primary to-blue-600 shadow-primary/20"
    };

    rsx! {
        button {
            class: "flex items-center gap-3 pl-2 pr-4 py-1.5 -ml-2 rounded-xl transition-all duration-200 hover:bg-[#1e2024] group outline-none",
            onclick: move |_| show_menu.toggle(),
            div { class: "h-9 w-9 rounded-xl flex items-center justify-center shadow-lg transition-colors {icon_bg}",
                span { class: "material-symbols-outlined text-white text-[20px]",
                    if is_terminal { "terminal" } else { "list_alt" }
                }
            }
            div { class: "flex flex-col items-start whitespace-nowrap",
                div { class: "flex items-center gap-1.5",
                    h1 { class: "text-lg font-bold tracking-tight leading-none text-white group-hover:text-primary transition-colors",
                        if is_terminal { "Terminal" } else { "Monitor" }
                    }
                    span { class: "material-symbols-outlined text-gray-500 text-[16px] transition-transform duration-200 group-hover:text-gray-300",
                        if show_menu() { "expand_less" } else { "expand_more" }
                    }
                }
                span { class: "text-[10px] font-medium text-gray-400 tracking-wider",
                    "{APP_SUBTITLE}"
                }
            }
        }
    }
}

#[component]
pub fn Header() -> Element {
    let show_menu = use_signal(|| false);

    rsx! {
        header { class: "shrink-0 h-14 p-2 flex items-center z-50 relative border-b border-[#2a2e33] bg-[#0d0f10]",
            div { class: "flex gap-3 items-center w-full min-w-[600px]",
                div { class: "flex-[1.3] relative min-w-0",
                    BrandTriggerButton { show_menu }
                    if show_menu() {
                        ModeDropdownMenu { show_menu }
                    }
                }

                div { class: "w-px h-8 bg-[#2a2e33]" }
                div { class: "flex-1 flex items-center justify-end min-w-0 pr-1",
                    ConnectionControl {}
                }
            }
        }
    }
}
