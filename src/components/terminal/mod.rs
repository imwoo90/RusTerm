//! # Terminal Emulation Subsystem (index.md)
//!
//! ## Overview
//! Embeds full VT100/ANSI terminal emulation via WebAssembly bindings to xterm.js v6.0, managing input throttling buffers, periodic screen writes, and window resizing.
//!
//! ## Submodules
//! - [`hooks`]: Terminal lifecycle hooks managing xterm.js instance initialization and resizing.
//! - [`toolbar`]: Action bar hosting clear buffer, copy output, and auto-scroll controls.
//!
//! ## Search Tags
//! #terminal, #xtermjs, #vt100, #ansi, #emulation

pub mod hooks;
pub mod toolbar;

use crate::components::ui::buttons::ResumeScrollButton;
use crate::components::ui::console::ConsoleFrame;
use crate::state::AppState;
use crate::utils::terminal_bindings::{Terminal, XtermFitAddon};
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::window;

pub use toolbar::TerminalToolbar;

/// RAII wrapper for Terminal that disposes it on drop
pub struct AutoDisposeTerminal(pub Terminal);

#[component]
pub fn TerminalView(term_instance: Signal<Option<AutoDisposeTerminal>>) -> Element {
    let app_state = use_context::<AppState>();

    rsx! {
        ConsoleFrame {
            TerminalToolbar { term_instance }
            Xterm { term_instance }
            if !*app_state.terminal.autoscroll.read() {
                ResumeScrollButton {
                    onclick: move |_| {
                        if let Some(term) = term_instance.read().as_ref() {
                            term.scroll_to_bottom();
                        }
                    },
                }
            }
        }
    }
}

impl Drop for AutoDisposeTerminal {
    fn drop(&mut self) {
        self.0.dispose();
    }
}

impl std::ops::Deref for AutoDisposeTerminal {
    type Target = Terminal;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct XtermProps {
    term_instance: Signal<Option<AutoDisposeTerminal>>,
}

fn use_options_sync(
    state: &AppState,
    term_instance: Signal<Option<AutoDisposeTerminal>>,
    fit_addon: Signal<Option<XtermFitAddon>>,
) {
    let font_size_sig = state.ui.font_size;
    let scrollback_sig = state.terminal.scrollback;
    use_effect(move || {
        let font_size = *font_size_sig.read();
        let scrollback = *scrollback_sig.read();

        if let Some(term) = term_instance.read().as_ref() {
            let options = term.options();
            options.set_font_size(font_size);
            options.set_scrollback(scrollback);

            if let Some(fit) = fit_addon.read().as_ref() {
                fit.fit();
            }
        }
    });
}

fn use_terminal_writer(
    mut lines_signal: Signal<usize>,
    term_instance: Signal<Option<AutoDisposeTerminal>>,
    aggregation_buffer: Signal<Rc<RefCell<Vec<u8>>>>,
) {
    use_resource(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(100).await;
            if let Some(term) = term_instance.read().as_ref() {
                let buffer_rc = aggregation_buffer.read().clone();
                let mut data_vec = buffer_rc.borrow_mut();
                if !data_vec.is_empty() {
                    let chunk = std::mem::take(&mut *data_vec);
                    drop(data_vec);
                    let array = js_sys::Uint8Array::from(chunk.as_slice());
                    term.write_chunk(&array);

                    let lines = term.buffer().active().length();
                    *lines_signal.write() = lines as usize;
                }
            }
        }
    });
}

fn use_serial_sender(
    port_sig: Signal<Option<web_sys::SerialPort>>,
    send_buffer: Signal<Rc<RefCell<Vec<u8>>>>,
) {
    use_resource(move || async move {
        loop {
            gloo_timers::future::TimeoutFuture::new(16).await;
            let data = {
                let buffer_rc = send_buffer.read().clone();
                let mut buf = buffer_rc.borrow_mut();
                if buf.is_empty() {
                    continue;
                }
                std::mem::take(&mut *buf)
            };
            if let Some(port) = port_sig.peek().clone() {
                if let Err(e) = crate::utils::serial_api::send_data(&port, &data).await {
                    web_sys::console::error_1(&e);
                }
            }
        }
    });
}

#[component]
pub fn Xterm(props: XtermProps) -> Element {
    let mut terminal_div = use_signal(|| None::<web_sys::HtmlElement>);
    let term_instance = props.term_instance;
    let fit_addon = use_signal(|| None::<XtermFitAddon>);
    let state = use_context::<AppState>();

    let aggregation_buffer = use_signal(|| Rc::new(RefCell::new(Vec::<u8>::new())));
    let send_buffer = use_signal(|| Rc::new(RefCell::new(Vec::<u8>::new())));
    let resize_listener = use_signal(|| None::<gloo_events::EventListener>);

    use_effect(move || {
        if let Some(div) = terminal_div.read().as_ref() {
            if term_instance.read().is_some() {
                return;
            }
            hooks::setup_terminal(
                div,
                state,
                send_buffer(),
                aggregation_buffer(),
                term_instance,
                resize_listener,
                fit_addon,
            );
        }
    });

    use_options_sync(&state, term_instance, fit_addon);

    use_effect(move || {
        let _ = state.terminal.received_data.read();
        let data = state.terminal.take_data();
        if !data.is_empty() {
            aggregation_buffer.read().borrow_mut().extend(data);
        }
    });

    use_terminal_writer(state.terminal.lines, term_instance, aggregation_buffer);
    use_serial_sender(state.conn.port, send_buffer);

    rsx! {
        div {
            class: "flex-1 w-full bg-transparent overflow-hidden pl-2",
            id: "xterm-container",
            onmounted: move |_| {
                if let Some(element) = window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .get_element_by_id("xterm-container")
                {
                    if let Ok(html_elem) = element.dyn_into::<web_sys::HtmlElement>() {
                        terminal_div.set(Some(html_elem));
                    }
                }
            },
        }
    }
}
