//! Line ending selector component for serial communication.
//!
//! Renders horizontal segmented buttons for switching between NONE/RAW, LF, CR, and CRLF line endings.
//! Provides distinct labels and active styles depending on whether it is configured for RX or TX streams.

use crate::state::LineEnding;
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
