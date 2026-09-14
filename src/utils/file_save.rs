//! # Browser File Download Utility
//!
//! ## Overview
//! Triggers browser file downloads via DOM Blob URL creation and temporary link clicks for saving exported log sessions to user disk.
//!
//! ## Search Tags
//! #file-save, #download, #blob, #export, #browser-fs

use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/public/assets/js/file_save.js")]
extern "C" {
    pub fn save_stream_to_disk(stream: JsValue);
    pub fn save_terminal_history(terminal: &JsValue);
}
