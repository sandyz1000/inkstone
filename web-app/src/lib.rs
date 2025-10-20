mod app;
mod components;
mod viewer;
// mod engine;
// mod wasm; // Old wasm module not compatible with Interactive trait
mod utils;
mod backend;
mod pdf_app;
mod interactive_app;

pub use app::App;
pub use interactive_app::InteractiveApp;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() {
    // Set up panic hook for better error messages
    console_error_panic_hook::set_once();

    // Initialize logging
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).expect("Failed to initialize logger");

    log::info!("🚀 Inkstone PDF Viewer starting (Interactive trait mode)...");
    log::info!("Built with Dioxus {} and WebAssembly", env!("CARGO_PKG_VERSION"));

    // Launch the Dioxus app with Interactive trait implementation
    dioxus::launch(interactive_app::InteractiveApp);
}
