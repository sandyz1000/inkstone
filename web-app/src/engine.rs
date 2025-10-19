use pdf::file::SyncCache;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ HtmlCanvasElement, WebGl2RenderingContext };
use std::sync::Arc;

use pathfinder_webgl::WebGlDevice;
use pathfinder_renderer::{
    gpu::{
        options::{ DestFramebuffer, RendererMode, RendererOptions, RendererLevel },
        renderer::Renderer,
    },
    options::{ BuildOptions, RenderTransform },
    concurrent::executor::SequentialExecutor,
};
use pathfinder_geometry::{
    vector::{ Vector2F, Vector2I },
    rect::{ RectF, RectI },
    transform2d::Transform2F,
};
use pathfinder_color::ColorF;
use pathfinder_resources::embedded::EmbeddedResourceLoader;
use inkrender::{ Cache as RenderCache, SceneBackend, page_bounds, render_page };
use pdf::file::{ File as PdfFile, FileOptions, NoLog };
use pdf::any::AnySync;
use pdf::PdfError;
use pdf::object::PlainRef;

pub struct PDFViewerEngine {
    canvas: HtmlCanvasElement,
    renderer: Renderer<WebGlDevice>,
    framebuffer_size: Vector2I,
    scale_factor: f32,
    pdf_file: Option<PdfFileWrapper>,
    render_cache: RenderCache,
}

struct PdfFileWrapper {
    file: PdfFile<
        Vec<u8>,
        Arc<SyncCache<PlainRef, Result<AnySync, Arc<PdfError>>>>,
        Arc<SyncCache<PlainRef, Result<Arc<[u8]>, Arc<PdfError>>>>,
        NoLog
    >,
}

impl PDFViewerEngine {
    pub fn new(canvas: HtmlCanvasElement, scale_factor: f32) -> Result<Self, JsValue> {
        log::info!("Initializing PDFViewerEngine with scale_factor: {}", scale_factor);

        // Get WebGL2 context
        let context = canvas
            .get_context("webgl2")?
            .ok_or_else(|| JsValue::from_str("Failed to get WebGL2 context"))?
            .dyn_into::<WebGl2RenderingContext>()?;

        // Set initial canvas size
        let css_width = 800.0;
        let css_height = 1000.0;
        let physical_width = (css_width * scale_factor).ceil() as u32;
        let physical_height = (css_height * scale_factor).ceil() as u32;

        canvas.set_width(physical_width);
        canvas.set_height(physical_height);

        let style = canvas.style();
        style
            .set_property("width", &format!("{}px", css_width))
            .map_err(|e| JsValue::from_str(&format!("Failed to set canvas width: {:?}", e)))?;
        style
            .set_property("height", &format!("{}px", css_height))
            .map_err(|e| JsValue::from_str(&format!("Failed to set canvas height: {:?}", e)))?;

        let framebuffer_size = Vector2I::new(physical_width as i32, physical_height as i32);

        // Create renderer
        let resource_loader = EmbeddedResourceLoader::new();
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

        let renderer = Renderer::new(device, &resource_loader, render_mode, render_options);

        log::info!("Renderer created successfully");

        Ok(Self {
            canvas,
            renderer,
            framebuffer_size,
            scale_factor,
            pdf_file: None,
            render_cache: RenderCache::new(),
        })
    }

    pub fn load_pdf_from_bytes(&mut self, data: Vec<u8>) -> Result<usize, String> {
        log::info!("Loading PDF from {} bytes", data.len());

        // Parse PDF file
        let file = FileOptions::cached()
            .load(data)
            .map_err(|e| format!("Failed to parse PDF: {:?}", e))?;

        let num_pages = file.num_pages() as usize;
        log::info!("PDF loaded successfully with {} pages", num_pages);

        self.pdf_file = Some(PdfFileWrapper { file });
        Ok(num_pages)
    }

    pub fn render_page(&mut self, page_num: usize, zoom: f32) -> Result<(), String> {
        log::info!("Rendering page {} with zoom {:.2}", page_num, zoom);

        let pdf_wrapper = self.pdf_file.as_ref().ok_or_else(|| "No PDF file loaded".to_string())?;

        // Get the page
        let page = pdf_wrapper.file
            .get_page((page_num - 1) as u32)
            .map_err(|e| format!("Failed to get page: {:?}", e))?;

        // Get page bounds
        let bounds = page_bounds(&page);
        log::info!("Page bounds: {:?}", bounds);

        // Create scene
        let mut backend = SceneBackend::new(&mut self.render_cache);
        let resolver = pdf_wrapper.file.resolver();

        // Calculate transform for centering and zoom
        let page_size = bounds.size();
        let canvas_size = self.framebuffer_size.to_f32();

        // Scale to fit canvas while maintaining aspect ratio
        let scale_x = canvas_size.x() / page_size.x();
        let scale_y = canvas_size.y() / page_size.y();
        let fit_scale = scale_x.min(scale_y) * zoom;

        // Center the page
        let scaled_size = page_size * fit_scale;
        let offset = (canvas_size - scaled_size) * 0.5;

        let transform =
            Transform2F::from_translation(offset) *
            Transform2F::from_scale(Vector2F::splat(fit_scale)) *
            Transform2F::from_translation(-bounds.origin());

        // Render page to scene
        render_page(&mut backend, &resolver, &page, transform).map_err(|e|
            format!("Failed to render page: {:?}", e)
        )?;

        let mut scene = backend.finish();

        // Set view box to canvas size
        scene.set_view_box(RectF::new(Vector2F::default(), canvas_size));

        // Build and render
        let options = BuildOptions {
            transform: RenderTransform::Transform2D(Transform2F::default()),
            dilation: Vector2F::default(),
            subpixel_aa_enabled: true,
        };

        scene.build_and_render(&mut self.renderer, options, SequentialExecutor);

        log::info!("Page rendered successfully");
        Ok(())
    }

    pub fn resize(&mut self, css_width: f32, css_height: f32) {
        let physical_width = (css_width * self.scale_factor).ceil() as u32;
        let physical_height = (css_height * self.scale_factor).ceil() as u32;

        self.canvas.set_width(physical_width);
        self.canvas.set_height(physical_height);

        let style = self.canvas.style();
        let _ = style.set_property("width", &format!("{}px", css_width));
        let _ = style.set_property("height", &format!("{}px", css_height));

        self.framebuffer_size = Vector2I::new(physical_width as i32, physical_height as i32);
        // TODO: Pathfinder API no longer has set_main_framebuffer_size
        // The framebuffer size is now set through DestFramebuffer::Default in RendererOptions

        log::info!(
            "Canvas resized to {}x{} (physical: {}x{})",
            css_width,
            css_height,
            physical_width,
            physical_height
        );
    }

    pub fn set_scale_factor(&mut self, scale_factor: f32) {
        if (self.scale_factor - scale_factor).abs() > 0.01 {
            self.scale_factor = scale_factor;
            // Trigger resize to update physical dimensions
            let style = self.canvas.style();
            if
                let (Ok(width), Ok(height)) = (
                    style.get_property_value("width"),
                    style.get_property_value("height"),
                )
            {
                if
                    let (Some(w), Some(h)) = (
                        width.strip_suffix("px").and_then(|s| s.parse::<f32>().ok()),
                        height.strip_suffix("px").and_then(|s| s.parse::<f32>().ok()),
                    )
                {
                    self.resize(w, h);
                }
            }
        }
    }
}

// Convenience functions for use from JavaScript/Dioxus
#[wasm_bindgen]
pub fn create_pdf_viewer(canvas_id: &str) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
    let document = window.document().ok_or_else(|| JsValue::from_str("No document"))?;
    let canvas = document
        .get_element_by_id(canvas_id)
        .ok_or_else(|| JsValue::from_str(&format!("Canvas '{}' not found", canvas_id)))?
        .dyn_into::<HtmlCanvasElement>()?;

    let scale_factor = window.device_pixel_ratio() as f32;
    let _viewer = PDFViewerEngine::new(canvas, scale_factor)?;

    // Store in global for later access (in a real app, use proper state management)
    log::info!("PDF Viewer created successfully");
    Ok(())
}
