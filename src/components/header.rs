use crate::components::connection_control::ConnectionControl;
use dioxus::prelude::*;

#[component]
pub fn Header() -> Element {
    rsx! {
        header { class: "shrink-0 h-18 px-6 flex items-center justify-between z-20 relative border-b border-[#2a2e33] bg-[#0d0f10]",
            // Left: Brand
            div { class: "flex items-center gap-3",
                div { class: "h-9 w-9 rounded-xl bg-linear-to-br from-primary to-blue-600 flex items-center justify-center shadow-lg shadow-primary/20",
                    span { class: "material-symbols-outlined text-black text-[22px] font-bold",
                        "terminal"
                    }
                }
                div { class: "flex flex-col",
                    h1 { class: "text-lg font-bold tracking-tight leading-none text-white",
                        "Serial"
                    }
                    span { class: "text-[10px] font-medium text-gray-400 tracking-wider uppercase",
                        "Monitor v1.0"
                    }
                }
            }

            // Right: Controls
            ConnectionControl {}
        }
    }
}
