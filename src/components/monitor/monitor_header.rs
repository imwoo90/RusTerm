use crate::components::ui::console::{
    ConsoleSeparator, ConsoleToggleButton, UnifiedConsoleToolbar,
};
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn MonitorHeader(
    autoscroll: bool,
    count: usize,
    onexport: EventHandler<MouseEvent>,
    onclear: EventHandler<MouseEvent>,
    ontoggle_autoscroll: EventHandler<MouseEvent>,
) -> Element {
    let state = use_context::<AppState>();

    rsx! {
        UnifiedConsoleToolbar {
            left: rsx! {
                span { class: "text-[10px] text-gray-500 font-mono hidden sm:block", "[ LINES: {count} / OPFS ENABLED ]" }
                ConsoleSeparator {}
                div { class: "flex items-center gap-1",
                    ConsoleToggleButton {
                        icon: "schedule",
                        title: "Toggle Timestamps",
                        active: (state.ui.show_timestamps)(),
                        onclick: move |_| state.ui.toggle_timestamps(),
                    }
                    ConsoleToggleButton {
                        icon: "hexagon",
                        title: "Toggle Hex View",
                        active: (state.ui.is_hex_view)(),
                        onclick: move |_| state.ui.toggle_hex_view(),
                    }
                }
            },
            font_size: state.ui.font_size,
            is_autoscroll: autoscroll,
            is_tracking_interactive: true,
            on_toggle_autoscroll: move |evt| ontoggle_autoscroll.call(evt),
            on_clear: move |evt| onclear.call(evt),
            on_export: move |evt| onexport.call(evt),
            min_font_size: 8,
            max_font_size: 36,
        }
    }
}
