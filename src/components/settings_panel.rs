use crate::components::common::CustomSelect;
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn SettingsPanel() -> Element {
    let mut state = use_context::<AppState>();
    let is_open = (state.show_settings)();

    rsx! {
        div {
            class: "mx-5 bg-surface rounded-xl border border-white/10 shadow-2xl transition-all duration-300 z-50 absolute top-full left-0 right-0 h-min",
            class: if is_open { "opacity-100 visible p-4 mt-3 overflow-visible" } else { "max-h-0 opacity-0 invisible mt-2 overflow-hidden" },
            id: "settings-panel",
            style: "width: calc(100% - 2.5rem);",
            div { class: "grid grid-cols-2 gap-x-4 gap-y-3",
                div { class: "flex flex-col gap-1.5",
                    label { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest px-1",
                        "Data Bits"
                    }
                    CustomSelect {
                        options: vec!["5", "6", "7", "8"],
                        selected: state.data_bits,
                        onchange: move |val| state.data_bits.set(val),
                    }
                }
                div { class: "flex flex-col gap-1.5",
                    label { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest px-1",
                        "Stop Bits"
                    }
                    CustomSelect {
                        options: vec!["1", "1.5", "2"],
                        selected: state.stop_bits,
                        onchange: move |val| state.stop_bits.set(val),
                    }
                }
                div { class: "flex flex-col gap-1.5",
                    label { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest px-1",
                        "Parity"
                    }
                    CustomSelect {
                        options: vec!["None", "Even", "Odd", "Mark", "Space"],
                        selected: state.parity,
                        onchange: move |val| state.parity.set(val),
                    }
                }
                div { class: "flex flex-col gap-1.5",
                    label { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest px-1",
                        "Flow Control"
                    }
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
