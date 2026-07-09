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

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        RusTerm {}
    }
}
