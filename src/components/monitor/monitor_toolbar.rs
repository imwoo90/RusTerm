use crate::components::monitor::{HighlightButton, SearchBar, TransmitBar};
use dioxus::prelude::*;

#[component]
pub fn MonitorToolbar() -> Element {
    rsx! {
        div { class: "shrink-0 p-2 bg-background-dark relative z-40",
            div { class: "flex gap-2 h-10 items-stretch min-w-[600px]",
                HighlightButton {}
                SearchBar {}
                // --- Divider ---
                div { class: "w-px bg-[#2a2e33] my-2 mx-1" }
                TransmitBar {}
            }
        }
    }
}
