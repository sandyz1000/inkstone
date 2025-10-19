use dioxus::prelude::*;
use crate::app::AppState;
use crate::viewer::PDFRenderer;
use web_sys::HtmlCanvasElement;
use wasm_bindgen::JsCast;

#[component]
pub fn PDFCanvas(app_state: Signal<AppState>) -> Element {
    let mut canvas_ref = use_signal(|| None::<HtmlCanvasElement>);
    let mut renderer = use_signal(|| None::<PDFRenderer>);

    // Initialize renderer when canvas is ready
    use_effect(move || {
        if let Some(canvas) = canvas_ref.read().as_ref() {
            if renderer.read().is_none() {
                match PDFRenderer::new(canvas.clone()) {
                    Ok(r) => {
                        log::info!("PDF Renderer initialized");
                        renderer.set(Some(r));
                    }
                    Err(e) => {
                        log::error!("Failed to initialize renderer: {:?}", e);
                    }
                }
            }
        }
    });

    // Re-render when app state changes
    use_effect(move || {
        let state = app_state.read();
        if let Some(ref mut r) = *renderer.write() {
            if state.file_loaded {
                r.render_page(state.current_page, state.zoom_level);
            }
        }
    });

    rsx! {
        div {
            class: "pdf-canvas-wrapper",
            style: "position: relative; width: 100%; height: 100%; display: flex; justify-content: center; align-items: center;",
            
            canvas {
                id: "pdf-canvas",
                onmounted: move |evt| {
                    if let Some(element) = evt.data.downcast::<web_sys::Element>() {
                        if let Ok(canvas) = element.clone().dyn_into::<HtmlCanvasElement>() {
                            canvas_ref.set(Some(canvas));
                        }
                    }
                },
                style: "max-width: 100%; max-height: 100%; background: white;",
                width: "800",
                height: "1000",
            }
            
            // Show loading or placeholder
            if !app_state.read().file_loaded {
                div {
                    style: "position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); text-align: center; color: #999;",
                    
                    div {
                        style: "font-size: 48px; margin-bottom: 16px;",
                        "📄"
                    }
                    
                    h2 {
                        style: "font-size: 24px; font-weight: 400; margin-bottom: 8px;",
                        "No PDF loaded"
                    }
                    
                    p {
                        style: "font-size: 14px; color: #666;",
                        "Open a PDF file to start viewing"
                    }
                }
            }
        }
    }
}
