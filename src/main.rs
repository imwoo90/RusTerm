use dioxus::prelude::*;

mod components;
mod config;
mod hooks;
mod state;
pub mod types;
mod utils;
mod worker;
use components::rust_term::RusTerm;


fn main() {
    #[cfg(target_arch = "wasm32")]
    if crate::worker::lifecycle::start_worker() {
        return;
    }

    // Remove the loading screen from the DOM before mounting Dioxus
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Some(loader) = document.get_element_by_id("loading-screen") {
                    loader.remove();
                }
            }
        }
    }

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        RusTerm {}
    }
}
