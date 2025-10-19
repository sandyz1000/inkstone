use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ HtmlCanvasElement, WebGl2RenderingContext };
use pathfinder_webgl::WebGlDevice;
use pathfinder_renderer::{
    gpu::{ options::{ DestFramebuffer, RendererMode, RendererOptions }, renderer::Renderer },
    scene::Scene,
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

pub struct PDFRenderer {
    canvas: HtmlCanvasElement,
    renderer: Renderer<WebGlDevice>,
    framebuffer_size: Vector2I,
    resource_loader: EmbeddedResourceLoader,
}

impl PDFRenderer {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        // Get WebGL2 context
        let context = canvas
            .get_context("webgl2")?
            .ok_or_else(|| JsValue::from_str("Failed to get WebGL2 context"))?
            .dyn_into::<WebGl2RenderingContext>()?;

        // Set initial canvas size
        let width = 800;
        let height = 1000;
        canvas.set_width(width);
        canvas.set_height(height);

        let framebuffer_size = Vector2I::new(width as i32, height as i32);

        // Create renderer
        let resource_loader = EmbeddedResourceLoader::new();
        let device = WebGlDevice::new(context);

        let render_mode = RendererMode {
            level: pathfinder_renderer::gpu::options::RendererLevel::D3D9,
        };

        let render_options = RendererOptions {
            background_color: Some(ColorF::white()),
            dest: DestFramebuffer::Default {
                viewport: RectI::new(Vector2I::zero(), framebuffer_size),
                window_size: framebuffer_size,
            },
            show_debug_ui: false,
        };

        let renderer = Renderer::new(device, &resource_loader, render_mode, render_options);

        Ok(Self {
            canvas,
            renderer,
            framebuffer_size,
            resource_loader,
        })
    }

    pub fn render_page(&mut self, page_num: usize, zoom: f32) {
        log::info!("Rendering page {} with zoom {}", page_num, zoom);

        // Create a simple test scene for now
        let mut scene = Scene::new();
        let view_box = RectF::new(Vector2F::default(), self.framebuffer_size.to_f32());
        scene.set_view_box(view_box);

        // TODO: Integrate with pdf_view to render actual PDF content
        // For now, just render a placeholder

        let transform = Transform2F::from_scale(Vector2F::splat(zoom));
        let options = BuildOptions {
            transform: RenderTransform::Transform2D(transform),
            dilation: Vector2F::default(),
            subpixel_aa_enabled: false,
        };

        scene.build_and_render(&mut self.renderer, options, SequentialExecutor);

        log::info!("Page rendered successfully");
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.canvas.set_width(width);
        self.canvas.set_height(height);
        self.framebuffer_size = Vector2I::new(width as i32, height as i32);

        // Note: In the new pathfinder API, we need to recreate the renderer with new options
        // or handle this differently. For now, just update the framebuffer size.
        // The next render will use the updated size.
        log::info!("Framebuffer resized to {}x{}", width, height);
    }

    pub fn load_pdf(&mut self, _data: &[u8]) -> Result<(), String> {
        // TODO: Integrate with pdf crate to load and parse PDF
        log::info!("Loading PDF data...");
        Ok(())
    }
}
