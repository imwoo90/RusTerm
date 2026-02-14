use crate::components::ui::console::UnifiedConsoleToolbar;
use crate::state::AppState;
use dioxus::prelude::*;

#[component]
pub fn TerminalToolbar(term_instance: Signal<Option<super::AutoDisposeTerminal>>) -> Element {
    let state = use_context::<AppState>();

    rsx! {
        UnifiedConsoleToolbar {
            left: rsx! {
                // History Config & Line Count
                div { class: "flex items-center gap-2",
                    span { class: "text-[10px] text-gray-500 font-mono", "[ LINES: {state.terminal.lines} / HISTORY:" }
                    ScrollbackInput {}
                    span { class: "text-[10px] text-gray-500 font-mono", "]" }
                }
            },
            font_size: state.ui.font_size,
            is_autoscroll: *state.terminal.autoscroll.read(),
            is_tracking_interactive: false,
            on_toggle_autoscroll: |_| {}, // No-op
            on_clear: move |_| {
                state.terminal.clear();
                if let Some(term) = term_instance.read().as_ref() {
                    term.clear();
                }
            },
            on_export: move |_| {
                if let Some(term) = term_instance.read().as_ref() {
                    crate::utils::file_save::save_terminal_history(&(term.0).clone());
                }
            },
            min_font_size: 8,
            max_font_size: 36,
        }
    }
}

#[component]
fn ScrollbackInput() -> Element {
    let mut state = use_context::<AppState>();
    let mut input_value = use_signal(|| state.terminal.scrollback.read().to_string());

    let val_str = input_value.read();
    let is_valid = val_str
        .parse::<u32>()
        .map(|v| (1000..=99999).contains(&v))
        .unwrap_or(false);
    let border_class = if is_valid {
        "border-[#222629] focus:border-primary"
    } else {
        "border-red-500 focus:border-red-500"
    };

    rsx! {
        input {
            class: "w-20 h-4 px-1 bg-[#0b0c0d] border rounded text-[10px] text-gray-300 text-center focus:outline-none transition-colors {border_class}",
            r#type: "number",
            min: "1000",
            max: "99999",
            value: "{val_str}",
            oninput: move |e| {
                input_value.set(e.value());
                if let Ok(val) = e.value().parse::<u32>() {
                    if (1000..=99999).contains(&val) {
                        *state.terminal.scrollback.write() = val;
                    }
                }
            },
            onblur: move |_| {
                // Reset to current valid state on blur
                input_value.set(state.terminal.scrollback.read().to_string());
            },
        }
    }
}
