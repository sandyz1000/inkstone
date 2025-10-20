use std::sync::Arc;
use viewer::{ Interactive, Context, Emitter, Config };
use pathfinder_renderer::scene::Scene;
use pathfinder_geometry::{ vector::Vector2F, rect::RectF };
use inkrender::{ Cache as RenderCache, SceneBackend, page_bounds, render_page };
use pdf::file::{ File as PdfFile, FileOptions, NoLog, SyncCache };
use pdf::any::AnySync;
use pdf::PdfError;
use pdf::object::PlainRef;

use crate::backend::DioxusBackend;

/// Events for PDF viewer interactions
#[derive(Debug, Clone)]
pub enum ViewerEvent {
    NextPage,
    PrevPage,
    GotoPage(usize),
    ZoomIn,
    ZoomOut,
    SetZoom(f32),
}

/// PDF file type alias matching native-app pattern
type PdfFileType = PdfFile<
    Vec<u8>,
    Arc<SyncCache<PlainRef, Result<AnySync, Arc<PdfError>>>>,
    Arc<SyncCache<PlainRef, Result<Arc<[u8]>, Arc<PdfError>>>>,
    NoLog
>;

/// Main PDF viewer application for web (Dioxus)
pub struct PdfViewerApp {
    pdf_file: Option<PdfFileType>,
    render_cache: RenderCache,
    emitter: Option<Emitter<ViewerEvent>>,
}

impl PdfViewerApp {
    pub fn new() -> Self {
        Self {
            pdf_file: None,
            render_cache: RenderCache::new(),
            emitter: None,
        }
    }

    /// Load a PDF file from bytes
    pub fn load_pdf(&mut self, data: Vec<u8>) -> Result<usize, String> {
        let file = FileOptions::cached()
            .load(data)
            .map_err(|e| format!("Failed to load PDF: {:?}", e))?;

        let num_pages = file.num_pages() as usize;
        self.pdf_file = Some(file);

        Ok(num_pages)
    }

    /// Get PDF metadata
    pub fn get_title(&self) -> Option<String> {
        self.pdf_file.as_ref().and_then(|file| {
            file.trailer.info_dict
                .as_ref()
                .and_then(|info| info.title.as_ref())
                .map(|pdf_str| pdf_str.to_string())
                .and_then(|result| result.ok())
        })
    }

    /// Check if a PDF is loaded
    pub fn is_loaded(&self) -> bool {
        self.pdf_file.is_some()
    }
}

impl Interactive for PdfViewerApp {
    type Event = ViewerEvent;
    type Backend = DioxusBackend;

    fn scene(&mut self, ctx: &mut Context<Self::Backend>) -> Scene {
        let mut backend = SceneBackend::new(&mut self.render_cache);

        if let Some(ref file) = self.pdf_file {
            if let Ok(page) = file.get_page(ctx.page_nr as u32) {
                let bounds = page_bounds(&page);
                ctx.set_bounds(bounds);

                let transform = ctx.view_transform();
                let resolver = file.resolver();

                if let Err(e) = render_page(&mut backend, &resolver, &page, transform) {
                    log::error!("Failed to render page: {:?}", e);
                }
            }
        }

        let mut scene = backend.finish();
        scene.set_view_box(RectF::new(Vector2F::default(), ctx.window_size));
        scene
    }

    fn init(&mut self, ctx: &mut Context<Self::Backend>, sender: Emitter<Self::Event>) {
        self.emitter = Some(sender);

        // Set initial number of pages if PDF is loaded
        if let Some(ref file) = self.pdf_file {
            ctx.num_pages = file.num_pages() as usize;
        }
    }

    fn title(&self) -> String {
        match self.get_title() {
            Some(title) => format!("Inkstone - {}", title),
            None => "Inkstone PDF Viewer".to_string(),
        }
    }

    fn window_size_hint(&self) -> Option<Vector2F> {
        Some(Vector2F::new(1200.0, 800.0))
    }

    fn event(&mut self, ctx: &mut Context<Self::Backend>, event: Self::Event) {
        match event {
            ViewerEvent::NextPage => ctx.next_page(),
            ViewerEvent::PrevPage => ctx.prev_page(),
            ViewerEvent::GotoPage(page) => ctx.goto_page(page),
            ViewerEvent::ZoomIn => ctx.zoom_by(0.5),
            ViewerEvent::ZoomOut => ctx.zoom_by(-0.5),
            ViewerEvent::SetZoom(zoom) => ctx.set_zoom(zoom),
        }
    }

    fn cursor_moved(&mut self, _ctx: &mut Context<Self::Backend>, _pos: Vector2F) {
        // Handle cursor movement if needed for features like tooltips
    }
}

impl Default for PdfViewerApp {
    fn default() -> Self {
        Self::new()
    }
}
