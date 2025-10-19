mod app;
mod components;
mod viewer;
mod engine;
mod utils;

pub use app::App;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() {
    // Set up panic hook for better error messages
    console_error_panic_hook::set_once();

    // Initialize logging
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).expect("Failed to initialize logger");

    log::info!("🚀 Inkstone PDF Viewer starting...");
    log::info!("Built with Dioxus {} and WebAssembly", env!("CARGO_PKG_VERSION"));

    // Launch the Dioxus app
    dioxus::launch(app::App);
}
