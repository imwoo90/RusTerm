//! Serial port status and connection badge component.
//!
//! Visually displays the current serial device link status, providing color-coded status badges
//! and connection indicators to inform the user of port availability.

use crate::utils::SerialDeviceInfo;
use dioxus::prelude::*;

#[component]
pub fn PortStatus(device_info: Option<SerialDeviceInfo>) -> Element {
    rsx! {
        if let Some(info) = device_info {
            div {
                class: if info.label == "Simulation" {
                    "flex items-center gap-2 px-3 py-1.5 bg-[#16181a] rounded-lg border border-yellow-500/30 h-9 transition-colors shadow-sm"
                } else {
                    "flex items-center gap-2 px-3 py-1.5 bg-[#16181a] rounded-lg border border-emerald-500/30 h-9 transition-colors shadow-sm"
                },
                title: "{info.description}",
                span {
                    class: if info.label == "Simulation" {
                        "material-symbols-outlined text-yellow-500 text-[18px]"
                    } else {
                        "material-symbols-outlined text-emerald-500 text-[18px]"
                    },
                    if info.label == "Simulation" { "bug_report" } else { "usb" }
                }
                span {
                    class: if info.label == "Simulation" {
                        "text-xs font-bold text-yellow-400 font-mono tracking-tight"
                    } else {
                        "text-xs font-bold text-emerald-400 font-mono tracking-tight"
                    },
                    "{info.label}"
                }
            }
        } else {
            div {
                class: "flex items-center gap-2 px-3 py-1.5 bg-[#16181a] rounded-lg border border-[#2a2e33] h-9",
                title: "No serial device connected",
                span { class: "material-symbols-outlined text-gray-500 text-[18px]",
                    "usb_off"
                }
                span { class: "text-xs font-bold text-gray-500 font-mono", "No Device" }
            }
        }
    }
}
