use crate::worker::dispatcher;
use crate::worker::state::WorkerState;
use crate::worker::types::WorkerMsg;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

/// Starts the worker and sets up the message handling loop
#[wasm_bindgen]
pub fn start_worker() {
    if !js_sys::Reflect::has(&js_sys::global(), &"WorkerGlobalScope".into()).unwrap_or(false) {
        return;
    }

    spawn_local(async move {
        let state = match WorkerState::new().await {
            Ok(s) => Rc::new(RefCell::new(s)),
            Err(e) => {
                web_sys::console::error_1(&e);
                return;
            }
        };

        let onmessage = {
            let s_ptr = state.clone();
            Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
                dispatcher::handle_message(s_ptr.clone(), event.data());
            }) as Box<dyn FnMut(_)>)
        };

        let scope = state.borrow().scope.clone();
        scope.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        let mut last_count = 0;
        loop {
            gloo_timers::future::TimeoutFuture::new(crate::config::WORKER_STATUS_INTERVAL_MS).await;
            let current = state.borrow().proc.get_line_count();
            if current != last_count {
                last_count = current;
                state
                    .borrow()
                    .send_msg(WorkerMsg::TotalLines(current as usize));
            }
        }
    });
}

/// Gets the path to the application script
pub fn get_app_script_path() -> String {
    let window = web_sys::window().expect("no global window instance found");
    let document = window.document().expect("should have a document on window");
    if let Ok(scripts) = document.query_selector_all("script[type='module']") {
        for i in 0..scripts.length() {
            if let Some(node) = scripts.item(i) {
                let script: web_sys::HtmlScriptElement = node.unchecked_into();
                let src = script.src();
                let s = src.to_lowercase();
                if (s.contains("serial_monitor") || s.contains("web_serial_monitor"))
                    && !s.contains("snippets")
                    && s.ends_with(".js")
                {
                    return src;
                }
            }
        }
    }
    "./serial_monitor.js".into()
}
