use dioxus::prelude::*;

use super::console::Console;
use super::filter_section::FilterSection;
use super::footer::Footer;
use super::header::Header;
use super::settings_panel::SettingsPanel;

#[component]
pub fn SerialMonitor() -> Element {
    rsx! {
        div {
            class: "bg-background-light dark:bg-background-dark h-screen w-full flex flex-col font-display text-white overflow-hidden selection:bg-primary/30 selection:text-primary",

            input { checked: true, class: "peer hidden", id: "timestamp-toggle", "type": "checkbox" }
            input { checked: true, class: "peer hidden", id: "autoscroll-toggle", "type": "checkbox" }
            input { class: "peer hidden", id: "settings-panel-toggle", "type": "checkbox" }
            input { class: "peer hidden", id: "highlight-panel-toggle", "type": "checkbox" }

            Header {}
            SettingsPanel {}
            FilterSection {}
            Console {}
            Footer {}
        }
    }
}
