use dioxus::prelude::*;

#[component]
pub fn CustomSelect(
    options: Vec<&'static str>,
    selected: Signal<&'static str>,
    onchange: EventHandler<&'static str>,
    class: Option<&'static str>,
) -> Element {
    let mut is_open = use_signal(|| false);

    // Default class if none provided
    let base_class = class.unwrap_or("w-full");

    rsx! {
        div { class: "relative {base_class} group/select",
            // Trigger Button
            button {
                class: "w-full flex items-center justify-between bg-[#0d0f10] border border-[#2a2e33] rounded-lg text-xs font-bold text-gray-300 py-2 px-3 hover:bg-[#16181a] hover:border-primary/50 transition-all duration-200 outline-none focus:border-primary/50",
                onclick: move |e| {
                    e.stop_propagation();
                    is_open.set(!is_open());
                },
                span { "{selected}" }
                span {
                    class: "material-symbols-outlined text-[18px] text-gray-500 group-hover/select:text-primary transition-all duration-300",
                    class: if is_open() { "rotate-180 text-primary" } else { "" },
                    "expand_more"
                }
            }

            // Dropdown Menu
            if is_open() {
                // Backdrop to close when clicking outside
                div {
                    class: "fixed inset-0 z-40 cursor-default",
                    onclick: move |_| is_open.set(false)
                }

                div {
                    class: "absolute top-full left-0 right-0 mt-1 z-50 bg-[#16181a] border border-white/10 rounded-xl shadow-2xl py-1 overflow-hidden animate-in fade-in slide-in-from-top-2 duration-200",
                    for opt in options {
                        button {
                            class: "w-full text-left px-3 py-2 text-[11px] font-bold transition-all duration-150",
                            class: if opt == selected() { "bg-primary/20 text-primary" } else { "text-gray-400 hover:bg-white/5 hover:text-white" },
                            onclick: move |_| {
                                onchange.call(opt);
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
