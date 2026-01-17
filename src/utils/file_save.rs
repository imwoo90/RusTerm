use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/assets/js/file_save.js")]
extern "C" {
    pub fn save_stream_to_disk(stream: JsValue);
}
