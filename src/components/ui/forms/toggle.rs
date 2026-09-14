//! Animated toggle switch component for application settings.
//!
//! Provides an accessible, animated pill-style toggle switch for boolean settings.
//! Features active state glow effects and responsive transitions.

use dioxus::prelude::*;

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
