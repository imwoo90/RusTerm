use crate::state::LineEnding;
use crate::utils::format_hex_input;
use dioxus::prelude::*;

#[component]
pub fn LineEndSelector(
    label: &'static str,
    selected: LineEnding,
    onselect: EventHandler<LineEnding>,
    active_class: &'static str,
    is_rx: bool,
) -> Element {
    rsx! {
        div { class: "flex items-center gap-2",
            span { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest",
                "{label}"
            }
            div { class: "flex bg-[#0d0f10] p-0.5 rounded-lg border border-[#2a2e33]",
                for ending in [LineEnding::None, LineEnding::NL, LineEnding::CR, LineEnding::NLCR] {
                    button {
                        class: "px-2 py-1 rounded text-[10px] font-bold transition-all duration-200",
                        class: if selected == ending { "{active_class} border shadow-sm" } else { "text-gray-500 hover:text-white" },
                        onclick: move |_| onselect.call(ending),
                        match ending {
                            LineEnding::None => if is_rx { "RAW" } else { "NONE" }
                            LineEnding::NL => "LF",
                            LineEnding::CR => "CR",
                            LineEnding::NLCR => "CRLF",
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn CommandInputGroup(
    value: Signal<String>,
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
        div { class: "relative group flex-1",
            class: if show_line_ending_menu() { "z-50" },
            input {
                class: "w-full h-full min-h-[40px] bg-[#0d0f10] text-sm text-white placeholder-gray-600 px-4 rounded-lg border border-[#2a2e33] focus:border-primary/50 focus:shadow-glow outline-none transition-all font-mono",
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
            div { class: "absolute right-2 top-1/2 -translate-y-1/2 flex items-center gap-1",
                // Local Echo Toggle (Optional)
                if let Some(mut echo_signal) = echo {
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

                // Hex Input Toggle
                div { class: "relative group/hex",
                    button {
                        class: "w-8 h-8 rounded flex items-center justify-center transition-colors pb-1",
                        class: if is_hex() { "text-primary bg-primary/10 hover:bg-primary/20" } else { "text-gray-500 hover:text-gray-300 hover:bg-white/5" },
                        onclick: move |_| is_hex.set(!is_hex()),
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

                // Line Ending Dropdown
                div { class: "relative group/le",
                    button {
                        class: "w-8 h-8 rounded flex items-center justify-center transition-colors pb-1",
                        class: if show_line_ending_menu() { "text-gray-300 bg-white/10" } else if line_ending() != LineEnding::None { "text-primary bg-primary/10 hover:bg-primary/20" } else { "text-gray-500 hover:text-gray-300 hover:bg-white/5" },
                        onclick: move |_| show_line_ending_menu.set(!show_line_ending_menu()),
                        title: "Line Ending",
                        span { class: "material-symbols-outlined text-[20px]", "keyboard_return" }
                    }
                    div { class: "absolute -bottom-1.5 right-0 left-0 flex justify-center pointer-events-none",
                        span {
                            class: "text-[9px] font-bold scale-75 origin-top transition-colors",
                            class: if line_ending() != LineEnding::None { "text-primary" } else { "text-gray-500" },
                            match line_ending() {
                                LineEnding::None => "NONE",
                                LineEnding::NL => "LF",
                                LineEnding::CR => "CR",
                                LineEnding::NLCR => "CRLF",
                            }
                        }
                    }

                    if show_line_ending_menu() {
                        div { class: "absolute top-full right-0 mt-2 w-24 bg-[#16181a] border border-[#2a2e33] rounded-lg shadow-xl overflow-hidden z-50 flex flex-col py-1",
                            for ending in [LineEnding::None, LineEnding::NL, LineEnding::CR, LineEnding::NLCR] {
                                button {
                                    class: "px-3 py-2 text-[11px] font-mono text-left hover:bg-white/5 transition-colors flex items-center justify-between group/item",
                                    onclick: move |_| {
                                        line_ending.set(ending);
                                        show_line_ending_menu.set(false);
                                    },
                                    span { class: if line_ending() == ending { "text-primary font-bold" } else { "text-gray-400 group-hover/item:text-gray-300" },
                                        match ending {
                                            LineEnding::None => "NONE",
                                            LineEnding::NL => "LF",
                                            LineEnding::CR => "CR",
                                            LineEnding::NLCR => "CRLF",
                                        }
                                    }
                                    if line_ending() == ending {
                                        span { class: "material-symbols-outlined text-[12px] text-primary",
                                            "check"
                                        }
                                    }
                                }
                            }
                        }
                    }
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

    rsx! {
        div {
            class: "relative {class} group/select",
            class: if is_open() { "z-[600]" },
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
                span {
                    class: "material-symbols-outlined text-[18px] text-gray-500 group-hover/select:text-primary transition-all duration-300",
                    class: if is_open() { "rotate-180 text-primary" } else { "" },
                    "expand_more"
                }
            }

            if is_open() {
                div {
                    class: "fixed inset-0 z-[400] cursor-default",
                    onclick: move |_| is_open.set(false),
                }

                div { class: "absolute top-full left-0 right-0 mt-1 z-[500] bg-[#16181a] border border-white/10 rounded-xl shadow-2xl py-1 overflow-hidden animate-in fade-in slide-in-from-top-2 duration-200",
                    for opt in options {
                        button {
                            class: "w-full text-left px-3 py-2 text-[11px] font-bold transition-all duration-150",
                            class: if opt == selected { "bg-primary/20 text-primary" } else { "text-gray-400 hover:bg-white/5 hover:text-white" },
                            onclick: move |_| {
                                onchange.call(opt.to_string());
                                is_open.set(false);
                            },
                            "{opt}"
                        }
                    }
                }
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

    rsx! {
        div {
            class: "relative {class} group/select",
            class: if is_open() { "z-[600]" },
            div { class: if disabled { "w-full flex items-center justify-between bg-[#0d0f10] border border-[#2a2e33] rounded-lg py-0 px-0 opacity-50 cursor-not-allowed" } else { "w-full flex items-center justify-between bg-[#0d0f10] border border-[#2a2e33] rounded-lg py-0 px-0 hover:bg-[#16181a] hover:border-primary/50 transition-all duration-200 focus-within:border-primary/50" },

                // Input field for custom value
                input {
                    class: if disabled { "w-full bg-transparent border-none text-xs font-bold text-gray-300 py-2 pl-3 outline-none placeholder-gray-600 cursor-not-allowed" } else { "w-full bg-transparent border-none text-xs font-bold text-gray-300 py-2 pl-3 outline-none placeholder-gray-600" },
                    value: "{selected}",
                    disabled: "{disabled}",
                    oninput: move |evt| {
                        onchange.call(evt.value());
                    },
                    onfocus: move |_| is_open.set(false), // Close dropdown when typing
                }

                // Dropdown trigger button
                button {
                    class: if disabled { "flex items-center justify-center px-2 py-2 outline-none border-l border-[#2a2e33] cursor-not-allowed" } else { "flex items-center justify-center px-2 py-2 outline-none border-l border-[#2a2e33] cursor-pointer" },
                    disabled: "{disabled}",
                    onclick: move |e| {
                        if !disabled {
                            e.stop_propagation();
                            is_open.set(!is_open());
                        }
                    },
                    span {
                        class: "material-symbols-outlined text-[18px] text-gray-500 group-hover/select:text-primary transition-all duration-300",
                        class: if is_open() { "rotate-180 text-primary" } else { "" },
                        "expand_more"
                    }
                }
            }

            if is_open() {
                div {
                    class: "fixed inset-0 z-[400] cursor-default",
                    onclick: move |_| is_open.set(false),
                }

                div { class: "absolute top-full left-0 right-0 mt-1 z-[500] bg-[#16181a] border border-white/10 rounded-xl shadow-2xl py-1 overflow-hidden animate-in fade-in slide-in-from-top-2 duration-200 max-h-60 overflow-y-auto custom-scrollbar",
                    for opt in options {
                        button {
                            class: "w-full text-left px-3 py-2 text-[11px] font-bold transition-all duration-150",
                            class: if opt == selected { "bg-primary/20 text-primary" } else { "text-gray-400 hover:bg-white/5 hover:text-white" },
                            onclick: move |_| {
                                onchange.call(opt.to_string());
                                is_open.set(false);
                            },
                            "{opt}"
                        }
                    }
                }
            }
        }
    }
}

/// A toggle switch for boolean settings.
#[component]
pub fn ToggleSwitch(
    label: &'static str,
    active: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        button {
            class: "flex items-center cursor-pointer group gap-2",
            onclick: move |evt| onclick.call(evt),
            div { class: "relative flex items-center",
                div {
                    class: "w-7 h-3.5 rounded-full transition-all duration-200 border border-white/5",
                    class: if active { "bg-primary border-primary shadow-[0_0_8px_rgba(0,191,255,0.4)]" } else { "bg-[#2a2e33] group-hover:bg-[#34393e]" },
                }
                div {
                    class: "absolute left-0 w-3.5 h-3.5 rounded-full transition-all duration-200",
                    class: if active { "translate-x-3.5 bg-white" } else { "bg-gray-500" },
                }
            }
            span {
                class: "text-[10px] font-bold uppercase tracking-widest transition-colors leading-none",
                class: if active { "text-primary" } else { "text-gray-500 group-hover:text-gray-300" },
                "{label}"
            }
        }
    }
}
