//! # Serial Port Settings & Baud Rate Picker
//!
//! ## Overview
//! Provides the baud rate selection dropdown (`BaudRatePicker`) and modal configuration panel (`SettingsDropdown`)
//! for adjusting serial port parameters including data bits, stop bits, parity checking, and flow control.
//!
//! ## Search Tags
//! #settings, #baud-rate, #serial-parameters, #parity, #flow-control, #stop-bits

use crate::components::ui::{CustomInputSelect, CustomSelect};
use crate::hooks::use_serial_controller;
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn BaudRatePicker() -> Element {
    let state = use_context::<AppState>();
    let controller = use_serial_controller();

    rsx! {
        div { class: "w-32",
            CustomInputSelect {
                options: vec![
                    "1200",
                    "2400",
                    "4800",
                    "9600",
                    "19200",
                    "38400",
                    "57600",
                    "115200",
                    "230400",
                    "460800",
                    "921600",
                ],
                selected: (state.serial.baud_rate)().to_string(),
                onchange: move |val: String| {
                    if let Ok(b) = val.parse::<u32>() {
                        controller.change_baud_rate(b);
                    }
                },
                class: "w-full",
                disabled: (state.conn.is_busy)(),
            }
        }
    }
}

#[component]
fn SettingField(
    label: &'static str,
    options: Vec<&'static str>,
    selected: String,
    disabled: bool,
    onchange: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "flex flex-col gap-1.5",
            label { class: "text-[10px] font-bold text-gray-500 uppercase tracking-widest px-1", "{label}" }
            CustomSelect {
                options,
                selected,
                onchange: move |val| onchange.call(val),
                disabled,
            }
        }
    }
}

#[component]
pub fn SettingsDropdown(is_open: bool, onclose: EventHandler<()>) -> Element {
    let state = use_context::<AppState>();
    let disabled = state.conn.is_connected();

    rsx! {
        if is_open {
            div {
                class: "fixed inset-0 z-40 cursor-default",
                onclick: move |_| onclose.call(()),
            }
        }
        div {
            class: "absolute top-full right-6 mt-2 w-80 bg-[#16181a] rounded-xl border border-[#2a2e33] shadow-2xl transition-all duration-300 z-50 origin-top-right text-left",
            class: if is_open { "opacity-100 visible scale-100 translate-y-0 p-4" } else { "opacity-0 invisible scale-95 -translate-y-2 p-0 overflow-hidden h-0" },
            div { class: "grid grid-cols-2 gap-x-3 gap-y-4",
                SettingField {
                    label: "Data Bits",
                    options: vec!["5", "6", "7", "8"],
                    selected: (state.serial.data_bits)().to_string(),
                    disabled,
                    onchange: move |val: String| {
                        if let Ok(b) = val.parse::<u8>() {
                            state.serial.set_data_bits(b);
                        }
                    },
                }
                SettingField {
                    label: "Stop Bits",
                    options: vec!["1", "2"],
                    selected: if (state.serial.stop_bits)() == 1 { "1".to_string() } else { "2".to_string() },
                    disabled,
                    onchange: move |val: String| {
                        if let Ok(b) = val.parse::<u8>() {
                            state.serial.set_stop_bits(b);
                        }
                    },
                }
                SettingField {
                    label: "Parity",
                    options: vec!["None", "Even", "Odd"],
                    selected: match (state.serial.parity)() {
                        crate::state::Parity::None => "None".to_string(),
                        crate::state::Parity::Even => "Even".to_string(),
                        crate::state::Parity::Odd => "Odd".to_string(),
                    },
                    disabled,
                    onchange: move |val: String| {
                        let p = match val.as_str() {
                            "Even" => crate::state::Parity::Even,
                            "Odd" => crate::state::Parity::Odd,
                            _ => crate::state::Parity::None,
                        };
                        state.serial.set_parity(p);
                    },
                }
                SettingField {
                    label: "Flow Control",
                    options: vec!["None", "Hardware"],
                    selected: match (state.serial.flow_control)() {
                        crate::state::FlowControl::None => "None".to_string(),
                        crate::state::FlowControl::Hardware => "Hardware".to_string(),
                    },
                    disabled,
                    onchange: move |val: String| {
                        let f = match val.as_str() {
                            "Hardware" => crate::state::FlowControl::Hardware,
                            _ => crate::state::FlowControl::None,
                        };
                        state.serial.set_flow_control(f);
                    },
                }
            }
        }
    }
}
