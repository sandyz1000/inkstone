use std::path::Path;
use std::sync::Arc;

use pathfinder_geometry::rect::RectF;
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_renderer::scene::Scene;
use pathfinder_color::ColorF;
use pdf::any::AnySync;
use pdf::error::PdfError;
use pdf::file::{ File as PdfFile, FileOptions, NoLog, SyncCache };
use pdf::object::PlainRef;
use image::RgbaImage;

use inkrender::{ page_bounds, render_page, Cache as RenderCache, SceneBackend };
use rasterize::Rasterizer;

type PdfFileType = PdfFile<
    Vec<u8>,
    Arc<SyncCache<PlainRef, Result<AnySync, Arc<PdfError>>>>,
    Arc<SyncCache<PlainRef, Result<Arc<[u8]>, Arc<PdfError>>>>,
    NoLog
>;

/// PDF Renderer that handles loading and rendering PDF documents
pub struct PdfRenderer {
    file: Arc<PdfFileType>,
    num_pages: usize,
    cache: RenderCache,
}

impl PdfRenderer {
    /// Create a new PDF renderer from a file path
    pub fn new(path: &Path) -> Result<Self, String> {
        // Open the PDF file directly from path
        let file = FileOptions::cached()
            .open(path)
            .map_err(|e| format!("Failed to open PDF: {}", e))?;

        let num_pages = file.num_pages() as usize;

        Ok(Self {
            file: Arc::new(file),
            num_pages,
            cache: RenderCache::new(),
        })
    }

    /// Get the total number of pages
    pub fn num_pages(&self) -> usize {
        self.num_pages
    }

    /// Render a specific page to a Scene
    pub fn render_page(
        &mut self,
        page_num: usize,
        transform: Transform2F
    ) -> Result<Scene, String> {
        if page_num >= self.num_pages {
            return Err(format!("Page {} out of range (total pages: {})", page_num, self.num_pages));
        }

        // Get the page
        let page = self.file
            .get_page(page_num as u32)
            .map_err(|e| format!("Failed to get page: {}", e))?;

        // Create a scene backend
        let mut backend = SceneBackend::new(&mut self.cache);

        // Get the resolver
        let resolver = self.file.resolver();

        // Render the page
        render_page(&mut backend, &resolver, &page, transform).map_err(|e|
            format!("Failed to render page: {}", e)
        )?;

        Ok(backend.finish())
    }

    /// Render a specific page to an image (RGBA)
    pub fn render_page_to_image(
        &mut self,
        page_num: usize,
        dpi: f32,
    ) -> Result<RgbaImage, String> {
        let scale = Transform2F::from_scale(dpi / 25.4);
        let scene = self.render_page(page_num, scale)?;
        
        // Spawn a separate thread to do OpenGL rendering
        // This prevents conflicts with the main UI rendering thread
        let handle = std::thread::spawn(move || {
            let mut rasterizer = Rasterizer::new();
            rasterizer.rasterize(scene, Some(ColorF::white()))
        });
        
        // Wait for the rendering to complete
        handle.join()
            .map_err(|_| "Rendering thread panicked".to_string())
    }

    /// Get the bounding box of a page
    pub fn page_bounds(&self, page_num: usize) -> Result<RectF, String> {
        if page_num >= self.num_pages {
            return Err(format!("Page {} out of range (total pages: {})", page_num, self.num_pages));
        }

        let page = self.file
            .get_page(page_num as u32)
            .map_err(|e| format!("Failed to get page: {}", e))?;

        Ok(page_bounds(&page))
    }

    /// Get PDF metadata (title, author, etc.)
    pub fn get_title(&self) -> Option<String> {
        self.file.trailer.info_dict
            .as_ref()
            .and_then(|info| info.title.as_ref())
            .and_then(|p| p.to_string().ok())
    }

    /// Get PDF author
    pub fn get_author(&self) -> Option<String> {
        self.file.trailer.info_dict
            .as_ref()
            .and_then(|info| info.author.as_ref())
            .and_then(|p| p.to_string().ok())
    }

    /// Get PDF subject
    pub fn get_subject(&self) -> Option<String> {
        self.file.trailer.info_dict
            .as_ref()
            .and_then(|info| info.subject.as_ref())
            .and_then(|p| p.to_string().ok())
    }
}
