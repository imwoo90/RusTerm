use crate::components::monitor::{HighlightButton, SearchBar, TransmitBar};
use dioxus::prelude::*;

#[component]
pub fn MonitorToolbar() -> Element {
    let mut highlight_open = use_signal(|| false);

    rsx! {
        div {
            class: "shrink-0 p-2 bg-background-dark relative",
            class: if highlight_open() { "z-60" } else { "z-40" },
            div { class: "flex gap-2 h-10 items-stretch min-w-[600px]",
                HighlightButton { is_open: highlight_open }
                SearchBar {}
                // --- Divider ---
                div { class: "w-px bg-[#2a2e33] my-2 mx-1" }
                TransmitBar {}
            }
        }
    }
}
