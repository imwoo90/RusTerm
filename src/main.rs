//! # RusTerm Application Entry Point
//!
//! ## Overview
//! Bootstraps the Web Worker background thread, manages DOM splash screen transitions, and mounts the root Dioxus reactive UI tree for Web Serial communication.
//!
//! ## Submodules
//! - [`components`]: High-level UI component tree including headers, toolbars, monitors, and terminals.
//! - [`config`]: Global configuration constants, UI limits, and virtual scrolling thresholds.
//! - [`hooks`]: Reactive state hooks for Web Serial hardware streams and Web Worker IPC bridges.
//! - [`state`]: Application-wide reactive state management and global context signals.
//! - [`types`]: Core domain types, shared models, and serialization envelopes.
//! - [`utils`]: Utility functions for ANSI decoding, formatting, device IDs, and browser APIs.
//! - [`worker`]: High-throughput background Web Worker handling serial ingestion and OPFS persistence.
//!
//! ## Search Tags
//! #main, #entry-point, #dioxus, #web-worker, #web-serial

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
