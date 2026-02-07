use crate::state::AppState;
use crate::utils::terminal_bindings::{Terminal, XtermFitAddon};
use dioxus::prelude::*;
use js_sys::Uint8Array;
use wasm_bindgen::JsCast;
use web_sys::window;

#[component]
pub fn Xterm() -> Element {
    let mut terminal_div = use_signal(|| None::<web_sys::HtmlElement>);
    let mut term_instance = use_signal(|| None::<Terminal>);
    let state = use_context::<AppState>();

    use_effect(move || {
        if let Some(div) = terminal_div.read().as_ref() {
            if term_instance.read().is_some() {
                return;
            }

            // Check if window.Terminal exists
            let win = window().unwrap();
            let term_constructor = js_sys::Reflect::get(&win, &"Terminal".into()).unwrap();

            if term_constructor.is_undefined() {
                web_sys::console::error_1(&"xterm.js not loaded".into());
                return;
            }

            let options = js_sys::Object::new();
            js_sys::Reflect::set(&options, &"convertEol".into(), &true.into()).unwrap();
            js_sys::Reflect::set(
                &options,
                &"theme".into(),
                &serde_wasm_bindgen::to_value(&serde_json::json!({
                    "background": "#000000",
                    "foreground": "#ffffff"
                }))
                .unwrap(),
            )
            .unwrap();

            let term = Terminal::new(&options);

            // Load FitAddon
            let fit_addon = XtermFitAddon::new_fit();
            term.load_addon(&fit_addon.clone().into());

            term.open(div);

            // Initial write
            term.write("Hello from xterm.js via Dioxus!\r\n");

            term_instance.set(Some(term));

            // Initial fit
            let fit_initial: XtermFitAddon = fit_addon.clone().unchecked_into();
            gloo_timers::callback::Timeout::new(100, move || {
                fit_initial.fit();
            })
            .forget();

            // --- Debounced Resize Handler ---
            let win_resize = window().unwrap();
            let fit_for_resize = fit_addon.clone();

            // Shared state for debouncing
            let timeout_handle = std::rc::Rc::new(std::cell::Cell::new(None));

            let closure = wasm_bindgen::prelude::Closure::wrap(Box::new(move || {
                let fit: XtermFitAddon = fit_for_resize.clone().unchecked_into();
                let handle_cell = timeout_handle.clone();

                // Cancel previous timeout
                if let Some(h) = handle_cell.take() {
                    drop(h);
                }

                // Set debounce (150ms)
                let new_handle = gloo_timers::callback::Timeout::new(150, move || {
                    fit.fit();
                });
                handle_cell.set(Some(new_handle));
            }) as Box<dyn FnMut()>);

            win_resize
                .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget(); // Leak for the duration of the app for simplicity in this demo
        }
    });

    // Data consumption effect - run every time received_data changes
    use_effect(move || {
        // Track the signal by reading it
        let _ = state.terminal.received_data.read();

        let data = state.terminal.take_data();
        if !data.is_empty() {
            if let Some(term) = term_instance.read().as_ref() {
                let array = Uint8Array::from(data.as_slice());
                term.write_chunk(&array);
            }
        }
    });

    rsx! {
        div {
            class: "w-full h-full bg-black",
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
