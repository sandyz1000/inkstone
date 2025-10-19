use std::sync::Arc;

use log::info;
use pathfinder_geometry::vector::Vector2F;
use pathfinder_renderer::scene::Scene;
use pdf::any::AnySync;
use pdf::backend::Backend;
use pdf::error::PdfError;
use pdf::file::{ Cache as PdfCache, File as PdfFile, Log };
use inkrender::{ page_bounds, render_page, Cache, SceneBackend };

use viewer::{ Context, Emitter, Interactive, ViewBackend };
use crate::backend::GpuiBackend;

/// PDF viewer implementation that works with any backend
pub struct PdfView<B: Backend, OC, SC, L> {
    file: PdfFile<B, OC, SC, L>,
    num_pages: usize,
    cache: Cache,
}

impl<B, OC, SC, L> PdfView<B, OC, SC, L>
    where
        B: Backend + 'static,
        OC: PdfCache<Result<AnySync, Arc<PdfError>>> + 'static,
        SC: PdfCache<Result<Arc<[u8]>, Arc<PdfError>>> + 'static,
        L: Log
{
    pub fn new(file: PdfFile<B, OC, SC, L>) -> Self {
        PdfView {
            num_pages: file.num_pages() as usize,
            file,
            cache: Cache::new(),
        }
    }

    pub fn num_pages(&self) -> usize {
        self.num_pages
    }

    pub fn file(&self) -> &PdfFile<B, OC, SC, L> {
        &self.file
    }
}

impl<B, OC, SC, L> Interactive
    for PdfView<B, OC, SC, L>
    where
        B: Backend + 'static,
        OC: PdfCache<Result<AnySync, Arc<PdfError>>> + 'static,
        SC: PdfCache<Result<Arc<[u8]>, Arc<PdfError>>> + 'static,
        L: Log + 'static
{
    type Event = Vec<u8>;
    type Backend = GpuiBackend;

    fn title(&self) -> String {
        self.file.trailer.info_dict
            .as_ref()
            .and_then(|info| info.title.as_ref())
            .and_then(|p| p.to_string().ok())
            .unwrap_or_else(|| "PDF View".into())
    }

    fn init(&mut self, ctx: &mut Context<Self::Backend>, _sender: Emitter<Self::Event>) {
        ctx.num_pages = self.num_pages;

        // Set icon if logo is available
        if
            let Ok(img) = image::load_from_memory_with_format(
                include_bytes!("../../logo.png"),
                image::ImageFormat::Png
            )
        {
            ctx.set_icon(img.to_rgba8().into());
        }
    }

    fn scene(&mut self, ctx: &mut Context<Self::Backend>) -> Scene {
        info!("drawing page {}", ctx.page_nr());

        let page = self.file.get_page(ctx.page_nr as u32).unwrap();

        ctx.set_bounds(page_bounds(&page));

        let mut backend = SceneBackend::new(&mut self.cache);
        let resolver = self.file.resolver();
        render_page(&mut backend, &resolver, &page, ctx.view_transform()).unwrap();
        backend.finish()
    }

    fn cursor_moved(&mut self, _ctx: &mut Context<Self::Backend>, pos: Vector2F) {
        // Can be implemented for hover effects
    }
}
