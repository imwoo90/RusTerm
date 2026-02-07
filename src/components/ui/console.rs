use dioxus::prelude::*;

#[component]
pub fn ConsoleFrame(children: Element) -> Element {
    rsx! {
        main { class: "flex-1 min-h-0 mx-4 mb-0 mt-0 relative group/console",
            div { class: "absolute inset-0 bg-console-bg rounded-t-2xl border-t border-x border-[#222629] shadow-[inset_0_0_20px_rgba(0,0,0,0.8)] overflow-hidden flex flex-col",
                div { class: "absolute inset-0 scanlines opacity-20 pointer-events-none z-10" }
                {children}
            }
        }
    }
}

#[component]
pub fn ConsoleToolbar(left: Element, right: Element) -> Element {
    rsx! {
        div { class: "shrink-0 h-6 bg-[#16181a] border-b border-[#222629] flex items-center justify-between px-3 z-20 relative",
            div { class: "flex items-center gap-4", {left} }
            div { class: "flex items-center gap-2", {right} }
        }
    }
}

#[component]
pub fn ConsoleSeparator() -> Element {
    rsx! {
        div { class: "w-px h-3 bg-[#2a2e33]" }
    }
}

#[component]
pub fn ConsoleFontSizeControl(
    font_size: Signal<u32>,
    #[props(default = 8)] min_size: u32,
    #[props(default = 36)] max_size: u32,
) -> Element {
    rsx! {
        div { class: "flex items-center gap-1",
            button {
                class: "flex items-center justify-center w-5 h-5 rounded hover:bg-white/10 transition-colors text-gray-500 hover:text-white",
                onclick: move |_| {
                    let current = *font_size.read();
                    if current > min_size {
                        *font_size.write() = current - 1;
                    }
                },
                title: "Decrease Font Size",
                span { class: "text-[10px] font-bold", "-" }
            }
            span { class: "text-[10px] text-gray-500 font-mono w-6 text-center", "{font_size}px" }
            button {
                class: "flex items-center justify-center w-5 h-5 rounded hover:bg-white/10 transition-colors text-gray-500 hover:text-white",
                onclick: move |_| {
                    let current = *font_size.read();
                    if current < max_size {
                        *font_size.write() = current + 1;
                    }
                },
                title: "Increase Font Size",
                span { class: "text-[10px] font-bold", "+" }
            }
        }
    }
}

#[component]
pub fn ConsoleTrackingIndicator(
    is_tracking: bool,
    #[props(default = false)] interactive: bool,
    ontoggle: EventHandler<MouseEvent>,
) -> Element {
    let cursor_class = if interactive {
        "cursor-pointer"
    } else {
        "cursor-default"
    };

    rsx! {
        div {
            class: "h-full flex items-center px-3 border-l border-[#222629] group/tracking select-none {cursor_class}",
            onclick: move |evt| {
                if interactive {
                    ontoggle.call(evt)
                }
            },
            if is_tracking {
                div { class: "text-[9px] font-mono text-primary/60 uppercase tracking-widest flex items-center gap-1 group-hover/tracking:text-primary transition-colors",
                    span { class: "w-1 h-1 rounded-full bg-primary animate-pulse" }
                    "Tracking"
                }
            } else {
                div { class: "text-[9px] font-mono text-yellow-500/60 uppercase tracking-widest group-hover/tracking:text-yellow-500 transition-colors",
                    "Paused"
                }
            }
        }
    }
}

#[component]
pub fn ConsoleActionButton(
    icon: &'static str,
    title: &'static str,
    onclick: EventHandler<MouseEvent>,
    #[props(default = "hover:text-white")] hover_color_class: &'static str,
) -> Element {
    rsx! {
        button {
            class: "flex items-center justify-center w-5 h-5 rounded hover:bg-white/10 transition-colors text-gray-500 {hover_color_class}",
            onclick: move |evt| onclick.call(evt),
            title: "{title}",
            span { class: "material-symbols-outlined text-[14px]", "{icon}" }
        }
    }
}
#[component]
pub fn UnifiedConsoleToolbar(
    /// Content for the left side of the toolbar (e.g., line count, custom controls)
    left: Element,
    /// Font size signal
    font_size: Signal<u32>,
    /// Auto-scroll state
    is_autoscroll: bool,
    /// Whether the tracking indicator is clickable
    #[props(default = true)]
    is_tracking_interactive: bool,
    /// Handler for toggling auto-scroll
    on_toggle_autoscroll: EventHandler<MouseEvent>,
    /// Handler for clearing logs
    on_clear: EventHandler<MouseEvent>,
    /// Optional handler for exporting logs (if None, button is hidden)
    on_export: Option<EventHandler<MouseEvent>>,
    /// Min font size
    #[props(default = 8)]
    min_font_size: u32,
    /// Max font size
    #[props(default = 36)]
    max_font_size: u32,
) -> Element {
    rsx! {
        ConsoleToolbar {
            left: rsx! {
                {left}
            },
            right: rsx! {
                ConsoleTrackingIndicator {
                    is_tracking: is_autoscroll,
                    interactive: is_tracking_interactive,
                    ontoggle: move |evt| on_toggle_autoscroll.call(evt),
                }
                ConsoleSeparator {}
                ConsoleFontSizeControl { font_size, min_size: min_font_size, max_size: max_font_size }
                ConsoleSeparator {}
                ConsoleActionButton {
                    icon: "delete",
                    title: "Clear Logs",
                    onclick: move |evt| on_clear.call(evt),
                    hover_color_class: "hover:text-red-500",
                }
                if let Some(export_handler) = on_export {
                    ConsoleActionButton {
                        icon: "download",
                        title: "Export Logs",
                        onclick: move |evt| export_handler.call(evt),
                        hover_color_class: "hover:text-primary",
                    }
                }
            },
        }
    }
}
