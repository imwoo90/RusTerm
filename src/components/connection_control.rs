use crate::components::common::{CustomSelect, IconButton};
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn ConnectionControl() -> Element {
    let mut state = use_context::<AppState>();
    let is_open = (state.show_settings)();

    let settings_icon_class = if is_open {
        "text-[20px] transition-all duration-300 rotate-45"
    } else {
        "text-[20px] transition-all duration-300"
    };

    rsx! {
        div { class: "flex items-center gap-3 h-full",
            // Port Info
            div { class: "flex items-center gap-2 px-3 py-1.5 bg-[#16181a] rounded-lg border border-[#2a2e33] h-9",
                span { class: "material-symbols-outlined text-[#29A329] text-[18px]", "usb" }
                span { class: "text-xs font-bold text-gray-300 font-mono", "COM3" }
            }

            // Baud Rate
            div { class: "w-32",
                CustomSelect {
                    options: vec!["9600", "19200", "38400", "57600", "115200"],
                    selected: state.baud_rate,
                    onchange: move |val| state.baud_rate.set(val),
                    class: "w-full",
                }
            }

            // Settings Button
            IconButton {
                icon: "settings",
                active: is_open,
                class: "w-9 h-9 bg-[#16181a] border border-[#2a2e33] rounded-lg hover:border-primary/50 hover:text-white transition-colors",
                icon_class: settings_icon_class,
                onclick: move |_| {
                    let current = (state.show_settings)();
                    state.show_settings.set(!current);
                },
                title: "Settings",
            }

            // Connect Button
            button { class: "group relative flex items-center gap-2 bg-primary hover:bg-primary-hover border border-primary/50 pl-3 pr-4 py-1.5 rounded-lg transition-all duration-300 active:scale-95 shadow-lg shadow-primary/20 ml-2",
                div { class: "relative flex h-2 w-2",
                    span { class: "animate-ping absolute inline-flex h-full w-full rounded-full bg-white opacity-75" }
                    span { class: "relative inline-flex rounded-full h-2 w-2 bg-white" }
                }
                span { class: "text-xs font-bold text-black group-hover:text-black/80 transition-colors uppercase tracking-wide", "Connect" }
            }

            // Settings Dropdown Panel
            if is_open {
                div {
                    class: "fixed inset-0 z-40 cursor-default",
                    onclick: move |_| state.show_settings.set(false)
                }
            }
            div {
                class: "absolute top-full right-6 mt-2 w-80 bg-[#16181a] rounded-xl border border-[#2a2e33] shadow-2xl transition-all duration-300 z-50 origin-top-right text-left",
                class: if is_open { "opacity-100 visible scale-100 translate-y-0 p-4" } else { "opacity-0 invisible scale-95 -translate-y-2 p-0 overflow-hidden h-0" },
                div { class: "grid grid-cols-2 gap-x-3 gap-y-4",
                    div { class: "flex flex-col gap-1.5",
                        label { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest px-1", "Data Bits" }
                        CustomSelect {
                            options: vec!["5", "6", "7", "8"],
                            selected: state.data_bits,
                            onchange: move |val| state.data_bits.set(val),
                        }
                    }
                    div { class: "flex flex-col gap-1.5",
                        label { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest px-1", "Stop Bits" }
                        CustomSelect {
                            options: vec!["1", "1.5", "2"],
                            selected: state.stop_bits,
                            onchange: move |val| state.stop_bits.set(val),
                        }
                    }
                    div { class: "flex flex-col gap-1.5",
                        label { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest px-1", "Parity" }
                        CustomSelect {
                            options: vec!["None", "Even", "Odd", "Mark", "Space"],
                            selected: state.parity,
                            onchange: move |val| state.parity.set(val),
                        }
                    }
                    div { class: "flex flex-col gap-1.5",
                        label { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest px-1", "Flow Control" }
                        CustomSelect {
                            options: vec!["None", "Hardware", "Software"],
                            selected: state.flow_control,
                            onchange: move |val| state.flow_control.set(val),
                        }
                    }

                }
            }
        }
    }
}
