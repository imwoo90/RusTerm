//! Initialization and event lifecycle hooks for xterm.js terminal emulation.
//!
//! Configures terminal DOM mount, terminal visual options, fit addon resizing,
//! and throttled scroll offset listeners to track viewport bottom states.

use crate::state::AppState;
use crate::utils::terminal_bindings::*;
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::window;

fn build_terminal_options(state: &AppState) -> js_sys::Object {
    let options = js_sys::Object::new();
    let _ = js_sys::Reflect::set(&options, &"convertEol".into(), &true.into());
    let theme_val = serde_wasm_bindgen::to_value(&serde_json::json!({
        "background": "#000000",
        "foreground": "#ffffff"
    }))
    .unwrap_or(wasm_bindgen::JsValue::NULL);
    let _ = js_sys::Reflect::set(&options, &"theme".into(), &theme_val);
    let _ = js_sys::Reflect::set(
        &options,
        &"fontSize".into(),
        &(*state.ui.font_size.read()).into(),
    );
    let _ = js_sys::Reflect::set(
        &options,
        &"scrollback".into(),
        &(*state.terminal.scrollback.read()).into(),
    );
    options
}

fn attach_input_handler(term: &Terminal, send_buffer: Rc<RefCell<Vec<u8>>>) {
    let closure = wasm_bindgen::prelude::Closure::wrap(Box::new(move |data: String| {
        send_buffer.borrow_mut().extend(data.into_bytes());
    }) as Box<dyn FnMut(String)>);
    term.on_data(closure.as_ref().unchecked_ref());
    closure.forget();
}

fn attach_scroll_handler(term: Terminal, mut autoscroll_signal: Signal<bool>) {
    let term_for_scroll: Terminal = term.clone().unchecked_into();
    let last_update = Rc::new(RefCell::new(0.0));

    let on_scroll_closure = wasm_bindgen::prelude::Closure::wrap(Box::new(move |_| {
        let now = js_sys::Date::now();
        let mut last = last_update.borrow_mut();
        if now - *last < 100.0 {
            return;
        }
        *last = now;

        let buffer = term_for_scroll.buffer().active();
        let is_at_bottom = (buffer.base_y() - buffer.viewport_y()).abs() <= 1;

        if *autoscroll_signal.peek() != is_at_bottom {
            *autoscroll_signal.write() = is_at_bottom;
        }
    }) as Box<dyn FnMut(i32)>);
    term.on_scroll(on_scroll_closure.as_ref().unchecked_ref());
    on_scroll_closure.forget();
}

/// Initialize the xterm.js terminal instance
pub fn setup_terminal(
    div: &web_sys::HtmlElement,
    state: AppState,
    send_buffer: Rc<RefCell<Vec<u8>>>,
    _aggregation_buffer: Rc<RefCell<Vec<u8>>>,
    mut term_instance: Signal<Option<super::AutoDisposeTerminal>>,
    mut resize_listener: Signal<Option<gloo_events::EventListener>>,
    mut fit_addon_signal: Signal<Option<XtermFitAddon>>,
) {
    let win = match window() {
        Some(w) => w,
        None => return,
    };
    let term_constructor = match js_sys::Reflect::get(&win, &"Terminal".into()) {
        Ok(c) if !c.is_undefined() => c,
        _ => {
            web_sys::console::error_1(&"xterm.js not loaded".into());
            return;
        }
    };
    let _ = term_constructor;

    let options = build_terminal_options(&state);
    let term = Terminal::new(&options);
    let fit_addon = XtermFitAddon::new_fit();
    term.load_addon(&fit_addon);
    term.open(div);

    fit_addon_signal.set(Some(fit_addon.clone().unchecked_into()));
    attach_input_handler(&term, send_buffer);

    let term_for_instance: Terminal = term.clone().unchecked_into();
    term_instance.set(Some(super::AutoDisposeTerminal(term_for_instance)));

    let fit_initial: XtermFitAddon = fit_addon.clone().unchecked_into();
    spawn(async move {
        gloo_timers::future::TimeoutFuture::new(100).await;
        fit_initial.fit();
    });

    let fit_for_resize = fit_addon;
    let listener = gloo_events::EventListener::new(&win, "resize", move |_| {
        let fit: XtermFitAddon = fit_for_resize.clone().unchecked_into();
        fit.fit();
    });
    resize_listener.set(Some(listener));

    attach_scroll_handler(term, state.terminal.autoscroll);
}
