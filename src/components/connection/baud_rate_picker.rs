//! # Baud Rate Picker Dropdown
//!
//! ## Overview
//! Provides a customizable selection dropdown for standard serial baud rates (9600 to 921600 bps) and custom user-entered communication speeds.
//!
//! ## Search Tags
//! #baud-rate, #picker, #dropdown, #serial-speed

use crate::components::ui::CustomInputSelect;
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
