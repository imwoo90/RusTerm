use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type Terminal;

    #[wasm_bindgen(constructor)]
    pub fn new(options: &JsValue) -> Terminal;

    #[wasm_bindgen(method, js_name = open)]
    pub fn open(this: &Terminal, parent: &web_sys::HtmlElement);

    #[wasm_bindgen(method, js_name = write)]
    pub fn write(this: &Terminal, data: &str);

    #[wasm_bindgen(method, js_name = write)]
    pub fn write_chunk(this: &Terminal, data: &js_sys::Uint8Array);

    #[wasm_bindgen(method, js_name = loadAddon)]
    pub fn load_addon(this: &Terminal, addon: &JsValue);

    #[wasm_bindgen(js_namespace = ["FitAddon"], js_name = "FitAddon")]
    pub type XtermFitAddon;

    #[wasm_bindgen(constructor, js_namespace = ["FitAddon"], js_class = "FitAddon")]
    pub fn new_fit() -> XtermFitAddon;

    #[wasm_bindgen(method, js_name = fit)]
    pub fn fit(this: &XtermFitAddon);
}
