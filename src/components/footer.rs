use dioxus::prelude::*;

#[component]
pub fn Footer() -> Element {
    rsx! {
        footer { class: "shrink-0 p-5 pt-3 bg-background-dark border-t border-[#2a2e33] z-20 relative",
            div { class: "absolute top-0 left-0 right-0 h-px bg-linear-to-r from-transparent via-primary/20 to-transparent" }
            div { class: "flex flex-col gap-3",
                div { class: "flex items-center justify-between",
                    span { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest",
                        "Payload"
                    }
                    div { class: "flex bg-[#0d0f10] p-0.5 rounded-lg border border-[#2a2e33]",
                        button { class: "px-2 py-1 rounded text-[10px] font-bold text-gray-500 hover:text-white transition-colors",
                            "NONE"
                        }
                        button { class: "px-2 py-1 rounded text-[10px] font-bold text-gray-500 hover:text-white transition-colors",
                            "NL"
                        }
                        button { class: "px-2 py-1 rounded text-[10px] font-bold text-gray-500 hover:text-white transition-colors",
                            "CR"
                        }
                        button { class: "px-2 py-1 rounded bg-primary/20 text-primary border border-primary/20 shadow-sm text-[10px] font-bold transition-colors",
                            "NL+CR"
                        }
                    }
                }
                div { class: "flex gap-3 items-stretch h-12",
                    div { class: "relative flex-1",
                        input {
                            class: "w-full h-full bg-[#0d0f10] text-sm text-white placeholder-gray-600 px-4 rounded-xl border border-[#2a2e33] focus:border-primary/50 focus:shadow-glow outline-none shadow-inset-input transition-all",
                            placeholder: "Enter ASCII command...",
                            "type": "text",
                        }
                        div { class: "absolute right-3 top-1/2 -translate-y-1/2 text-gray-600 pointer-events-none",
                            span { class: "material-symbols-outlined text-[16px]", "keyboard" }
                        }
                    }
                    button { class: "h-full aspect-square bg-primary text-surface rounded-xl flex items-center justify-center hover:bg-white transition-all hover:shadow-[0_0_15px_rgba(255,255,255,0.4)] active:scale-95 group",
                        span { class: "material-symbols-outlined text-[22px] group-hover:rotate-45 transition-transform duration-300",
                            "send"
                        }
                    }
                }
            }
        }
    }
}
