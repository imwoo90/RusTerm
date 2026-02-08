use crate::worker::dispatcher;
use crate::worker::state::WorkerState;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

/// Starts the worker and sets up the message handling loop
pub fn start_worker() -> bool {
    if !js_sys::Reflect::has(&js_sys::global(), &"WorkerGlobalScope".into()).unwrap_or(false) {
        return false;
    }

    spawn_local(async move {
        let state = match WorkerState::new().await {
            Ok(s) => Rc::new(RefCell::new(s)),
            Err(e) => {
                web_sys::console::error_1(&e);
                return;
            }
        };

        WorkerState::start_periodic_updates(state.clone());

        let onmessage = {
            let s_ptr = state.clone();
            Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
                dispatcher::handle_message(s_ptr.clone(), event.data());
            }) as Box<dyn FnMut(_)>)
        };

        let scope = state.borrow().scope.clone();
        scope.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        web_sys::console::log_1(&"Worker: Started and listening".into());
    });

    true
}

/// Gets the path to the application script by searching the DOM
pub fn get_app_script_path() -> String {
    let document = web_sys::window()
        .and_then(|w| w.document())
        .expect("No document");
    let pkg_name = env!("CARGO_PKG_NAME");

    // Select only relevant tags that contain the package name in their source/href
    let selector = format!("script[src*='{pkg_name}'], link[href*='{pkg_name}'][rel='preload']");

    document
        .query_selector_all(&selector)
        .ok()
        .and_then(|nodes| {
            (0..nodes.length()).find_map(|i| {
                let node = nodes.item(i)?;

                // Efficiently get the source URL without unnecessary clones
                let src = node
                    .dyn_ref::<web_sys::HtmlScriptElement>()
                    .map(|s| s.src())
                    .or_else(|| node.dyn_ref::<web_sys::HtmlLinkElement>().map(|l| l.href()))?;

                // Confirm it's the main JS bundle (ends with .js, ignoring query params)
                let is_main_bundle =
                    src.split('?').next()?.ends_with(".js") && !src.contains("snippets");

                is_main_bundle.then_some(src)
            })
        })
        .unwrap_or_else(|| format!("./{pkg_name}.js"))
}
