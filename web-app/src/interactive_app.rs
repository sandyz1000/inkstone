use dioxus::prelude::*;
use web_sys::{ HtmlCanvasElement, WebGl2RenderingContext };
use wasm_bindgen::JsCast;
use std::rc::Rc;
use std::cell::RefCell;

use pathfinder_webgl::WebGlDevice;
use pathfinder_renderer::{
    gpu::{
        options::{ DestFramebuffer, RendererMode, RendererOptions, RendererLevel },
        renderer::Renderer,
    },
    options::{ BuildOptions, RenderTransform },
    concurrent::executor::SequentialExecutor,
};
use pathfinder_geometry::{ vector::{ Vector2F, Vector2I }, rect::RectI, transform2d::Transform2F };
use pathfinder_color::ColorF;
use pathfinder_resources::embedded::EmbeddedResourceLoader;

use viewer::{ Context, Config, Emitter, Interactive };
use crate::backend::DioxusBackend;
use crate::pdf_app::{ PdfViewerApp, ViewerEvent };

/// State for the WebGL PDF renderer
pub struct WebGlRenderer {
    renderer: Renderer<WebGlDevice>,
    viewer_app: PdfViewerApp,
    viewer_context: Context<DioxusBackend>,
}

impl WebGlRenderer {
    pub fn new(
        canvas: &HtmlCanvasElement,
        scale_factor: f32
    ) -> Result<Self, wasm_bindgen::JsValue> {
        // Get WebGL2 context
        let context = canvas
            .get_context("webgl2")?
            .ok_or_else(|| wasm_bindgen::JsValue::from_str("Failed to get WebGL2 context"))?
            .dyn_into::<WebGl2RenderingContext>()?;

        let width = 1200;
        let height = 800;
        canvas.set_width(width);
        canvas.set_height(height);

        let framebuffer_size = Vector2I::new(width as i32, height as i32);

        // Create renderer
        let renderer_resource_loader = EmbeddedResourceLoader::new();
        let device = WebGlDevice::new(context);

        let render_mode = RendererMode {
            level: RendererLevel::D3D9,
        };

        let render_options = RendererOptions {
            background_color: Some(ColorF::new(0.95, 0.95, 0.95, 1.0)),
            dest: DestFramebuffer::Default {
                viewport: RectI::new(Vector2I::zero(), framebuffer_size),
                window_size: framebuffer_size,
            },
            show_debug_ui: false,
        };

        let renderer = Renderer::new(
            device,
            &renderer_resource_loader,
            render_mode,
            render_options
        );

        // Create viewer context with a separate resource loader for Config
        let config_resource_loader = EmbeddedResourceLoader::new();
        let config = Rc::new(Config::new(Box::new(config_resource_loader)));
        let backend = DioxusBackend::new();
        let mut viewer_context = Context::new(config, backend);
        viewer_context.set_window_size(framebuffer_size.to_f32());
        viewer_context.set_scale_factor(scale_factor);

        // Create viewer app
        let mut viewer_app = PdfViewerApp::new();

        // Initialize with a dummy emitter (will be replaced when we have actual event handling)
        let emitter = Emitter { inner: ViewerEvent::NextPage };
        viewer_app.init(&mut viewer_context, emitter);

        Ok(Self {
            renderer,
            viewer_app,
            viewer_context,
        })
    }

    pub fn load_pdf(&mut self, data: Vec<u8>) -> Result<usize, String> {
        let num_pages = self.viewer_app.load_pdf(data)?;
        self.viewer_context.num_pages = num_pages;
        self.viewer_context.request_redraw();
        Ok(num_pages)
    }

    pub fn render(&mut self) {
        // Generate scene using Interactive trait
        let mut scene = self.viewer_app.scene(&mut self.viewer_context);

        // Build and render the scene
        let options = BuildOptions {
            transform: RenderTransform::Transform2D(Transform2F::default()),
            dilation: Vector2F::default(),
            subpixel_aa_enabled: true,
        };

        scene.build_and_render(&mut self.renderer, options, SequentialExecutor);

        self.viewer_context.redraw_requested = false;
    }

    pub fn handle_event(&mut self, event: ViewerEvent) {
        self.viewer_app.event(&mut self.viewer_context, event);
        if self.viewer_context.redraw_requested {
            self.render();
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let new_size = Vector2F::new(width as f32, height as f32);
        self.viewer_context.set_window_size(new_size);

        log::info!("Resized to {}x{}", width, height);

        if self.viewer_context.redraw_requested {
            self.render();
        }
    }

    pub fn get_page_info(&self) -> (usize, usize) {
        (self.viewer_context.page_nr + 1, self.viewer_context.num_pages)
    }

    pub fn get_zoom(&self) -> f32 {
        self.viewer_context.scale
    }
}

/// Main application state
#[derive(Clone)]
struct AppState {
    current_page: usize,
    total_pages: usize,
    zoom: f32,
    file_loaded: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_page: 1,
            total_pages: 0,
            zoom: 1.0,
            file_loaded: false,
        }
    }
}

#[component]
pub fn InteractiveApp() -> Element {
    let mut app_state = use_signal(AppState::default);
    let renderer = use_signal(|| None::<Rc<RefCell<WebGlRenderer>>>);
    let canvas_id = "pdf-canvas";

    // Initialize renderer when component mounts - use_effect with async to ensure DOM is ready
    use_effect(move || {
        // Use a small delay to ensure the canvas is in the DOM
        let mut renderer_clone = renderer.clone();
        wasm_bindgen_futures::spawn_local(async move {
            // Small delay to let the DOM render
            gloo_timers::future::TimeoutFuture::new(50).await;

            let window = web_sys::window().expect("No window");
            let document = window.document().expect("No document");

            if let Some(canvas_elem) = document.get_element_by_id("pdf-canvas") {
                if let Ok(canvas) = canvas_elem.dyn_into::<web_sys::HtmlCanvasElement>() {
                    let scale_factor = window.device_pixel_ratio() as f32;

                    match WebGlRenderer::new(&canvas, scale_factor) {
                        Ok(gl_renderer) => {
                            log::info!("WebGL renderer created successfully");
                            *renderer_clone.write() = Some(Rc::new(RefCell::new(gl_renderer)));
                        }
                        Err(e) => log::error!("Failed to create WebGL renderer: {:?}", e),
                    }
                } else {
                    log::error!("Failed to cast canvas element");
                }
            } else {
                log::error!("Canvas element not found in DOM");
            }
        });
    });

    // File input handler using gloo-file
    let on_file_change = move |evt: Event<FormData>| {
        async move {
            if let Some(file_engine) = evt.files() {
                let files = file_engine.files();

                if let Some(file_name) = files.first() {
                    log::info!("Reading PDF file: {}", file_name);

                    // Read file using gloo-file
                    if let Some(data) = file_engine.read_file(file_name).await {
                        log::info!("File read successfully: {} bytes", data.len());

                        if let Some(renderer_ref) = renderer.read().as_ref() {
                            let mut renderer_mut = renderer_ref.borrow_mut();
                            match renderer_mut.load_pdf(data) {
                                Ok(num_pages) => {
                                    log::info!("PDF loaded with {} pages", num_pages);
                                    renderer_mut.render();

                                    let (current, total) = renderer_mut.get_page_info();
                                    app_state.write().current_page = current;
                                    app_state.write().total_pages = total;
                                    app_state.write().file_loaded = true;
                                }
                                Err(e) => log::error!("Failed to load PDF: {}", e),
                            }
                        }
                    } else {
                        log::error!("Failed to read file");
                    }
                }
            }
        }
    }; // Navigation handlers
    let handle_prev = move |_| {
        if let Some(renderer_ref) = renderer.read().as_ref() {
            let mut renderer_mut = renderer_ref.borrow_mut();
            renderer_mut.handle_event(ViewerEvent::PrevPage);
            let (current, _) = renderer_mut.get_page_info();
            app_state.write().current_page = current;
        }
    };

    let handle_next = move |_| {
        if let Some(renderer_ref) = renderer.read().as_ref() {
            let mut renderer_mut = renderer_ref.borrow_mut();
            renderer_mut.handle_event(ViewerEvent::NextPage);
            let (current, _) = renderer_mut.get_page_info();
            app_state.write().current_page = current;
        }
    };

    let handle_zoom_in = move |_| {
        if let Some(renderer_ref) = renderer.read().as_ref() {
            let mut renderer_mut = renderer_ref.borrow_mut();
            renderer_mut.handle_event(ViewerEvent::ZoomIn);
            app_state.write().zoom = renderer_mut.get_zoom();
        }
    };

    let handle_zoom_out = move |_| {
        if let Some(renderer_ref) = renderer.read().as_ref() {
            let mut renderer_mut = renderer_ref.borrow_mut();
            renderer_mut.handle_event(ViewerEvent::ZoomOut);
            app_state.write().zoom = renderer_mut.get_zoom();
        }
    };

    rsx! {
        div {
            class: "app-container",
            style: "display: flex; flex-direction: column; height: 100vh; width: 100vw; overflow: hidden; background: #1e1e1e; color: #e0e0e0;",
            
            // Header
            div {
                class: "header",
                style: "padding: 16px; background: #252526; border-bottom: 1px solid #3c3c3c; display: flex; justify-content: space-between; align-items: center;",
                
                h1 {
                    style: "margin: 0; font-size: 20px;",
                    "Inkstone PDF Viewer (Interactive)"
                }
                
                div {
                    style: "display: flex; gap: 12px; align-items: center;",
                    
                    label {
                        style: "cursor: pointer; padding: 8px 16px; background: #0e639c; border-radius: 4px; transition: background 0.2s;",
                        "Open PDF"
                        input {
                            id: "file-input",
                            r#type: "file",
                            accept: ".pdf",
                            style: "display: none;",
                            onchange: on_file_change,
                        }
                    }
                }
            }
            
            // Toolbar
            if app_state.read().file_loaded {
                div {
                    class: "toolbar",
                    style: "padding: 12px 16px; background: #2d2d2d; border-bottom: 1px solid #3c3c3c; display: flex; gap: 16px; align-items: center;",
                    
                    button {
                        onclick: handle_prev,
                        disabled: app_state.read().current_page <= 1,
                        style: "padding: 8px 16px; background: #0e639c; border-radius: 4px; cursor: pointer;",
                        "Previous"
                    }
                    
                    span {
                        "Page {app_state.read().current_page} / {app_state.read().total_pages}"
                    }
                    
                    button {
                        onclick: handle_next,
                        disabled: app_state.read().current_page >= app_state.read().total_pages,
                        style: "padding: 8px 16px; background: #0e639c; border-radius: 4px; cursor: pointer;",
                        "Next"
                    }
                    
                    div {
                        style: "margin-left: auto; display: flex; gap: 8px; align-items: center;",
                        
                        button {
                            onclick: handle_zoom_out,
                            style: "padding: 8px 16px; background: #0e639c; border-radius: 4px; cursor: pointer;",
                            "−"
                        }
                        
                        span {
                            "Zoom: {app_state.read().zoom:.1}"
                        }
                        
                        button {
                            onclick: handle_zoom_in,
                            style: "padding: 8px 16px; background: #0e639c; border-radius: 4px; cursor: pointer;",
                            "+"
                        }
                    }
                }
            }
            
            // Canvas container
            div {
                class: "canvas-container",
                style: "flex: 1; display: flex; justify-content: center; align-items: center; overflow: auto; background: #2d2d2d;",
                
                canvas {
                    id: "{canvas_id}",
                    style: "display: block; box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);",
                }
            }
        }
    }
}
