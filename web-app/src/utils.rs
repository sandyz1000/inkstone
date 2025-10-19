use wasm_bindgen::prelude::*;
use web_sys::{ Window, Document, HtmlElement };

pub fn window() -> Window {
    web_sys::window().expect("no global `window` exists")
}

pub fn document() -> Document {
    window().document().expect("should have a document on window")
}

pub fn body() -> HtmlElement {
    document().body().expect("document should have a body")
}

pub fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

pub fn scale_factor() -> f64 {
    window().device_pixel_ratio()
}

pub fn log(s: &str) {
    web_sys::console::log_1(&JsValue::from_str(s));
}
