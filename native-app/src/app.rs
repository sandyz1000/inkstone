use gpui::*;
use std::path::PathBuf;
use std::rc::Rc;
use pathfinder_geometry::vector::Vector2F;
use pathfinder_geometry::transform2d::Transform2F;
use pathfinder_renderer::scene::Scene;
use pathfinder_resources::embedded::EmbeddedResourceLoader;

use crate::renderer::PdfRenderer;
use crate::backend::GpuiBackend;
use viewer::{ Context as ViewerContext, Config, Interactive, Emitter };

/// Custom events for the PDF viewer
#[derive(Debug, Clone)]
pub enum ViewerEvent {
    LoadFile(String),
    PageChanged(usize),
    ZoomChanged(f32),
}

// Safety: ViewerEvent only contains owned data that can be sent between threads
unsafe impl Send for ViewerEvent {}

/// Main PDF Viewer Application State
/// Integrates viewer::Context with GPUI for PDF rendering
pub struct PdfViewerApp {
    /// Currently loaded PDF file path
    current_file: Option<PathBuf>,
    /// PDF renderer instance
    pdf_renderer: Option<PdfRenderer>,
    /// Viewer context managing navigation and zoom
    viewer_context: ViewerContext<GpuiBackend>,
    /// Error message if any
    error_message: Option<String>,
    /// Event emitter for viewer events
    emitter: Option<Emitter<ViewerEvent>>,
}

impl PdfViewerApp {
    /// Create a new PDF Viewer application
    pub fn new(_cx: &mut Context<Self>) -> Self {
        let backend = GpuiBackend::new();

        // Create config with embedded resource loader
        let resource_loader = Box::new(EmbeddedResourceLoader::new());
        let config = Rc::new(Config::new(resource_loader));

        let mut viewer_context = ViewerContext::new(config, backend);

        // Set initial window size
        viewer_context.window_size = Vector2F::new(800.0, 600.0);

        Self {
            current_file: None,
            pdf_renderer: None,
            viewer_context,
            error_message: None,
            emitter: None,
        }
    }

    /// Load a PDF file
    pub fn load_pdf(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        match PdfRenderer::new(&path) {
            Ok(renderer) => {
                let num_pages = renderer.num_pages();
                self.current_file = Some(path);
                self.pdf_renderer = Some(renderer);
                self.error_message = None;

                // Update viewer context with the number of pages
                self.viewer_context.num_pages = num_pages;
                self.viewer_context.goto_page(0);

                cx.notify();
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to load PDF: {}", e));
                self.pdf_renderer = None;
                cx.notify();
            }
        }
    }

    /// Navigate to next page
    pub fn next_page(&mut self, cx: &mut Context<Self>) {
        self.viewer_context.next_page();
        cx.notify();
    }

    /// Navigate to previous page
    pub fn prev_page(&mut self, cx: &mut Context<Self>) {
        self.viewer_context.prev_page();
        cx.notify();
    }

    /// Go to specific page (0-indexed)
    pub fn goto_page(&mut self, page: usize, cx: &mut Context<Self>) {
        self.viewer_context.goto_page(page);
        cx.notify();
    }

    /// Zoom in
    pub fn zoom_in(&mut self, cx: &mut Context<Self>) {
        self.viewer_context.zoom_by(1.2);
        cx.notify();
    }

    /// Zoom out
    pub fn zoom_out(&mut self, cx: &mut Context<Self>) {
        self.viewer_context.zoom_by(1.0 / 1.2);
        cx.notify();
    }

    /// Reset zoom to 100%
    pub fn reset_zoom(&mut self, cx: &mut Context<Self>) {
        self.viewer_context.set_zoom(1.0);
        cx.notify();
    }

    /// Get current page number (1-indexed for display)
    pub fn current_page_display(&self) -> usize {
        self.viewer_context.page_nr() + 1
    }

    /// Get total pages
    pub fn total_pages(&self) -> usize {
        self.viewer_context.num_pages
    }

    /// Get current zoom level
    pub fn zoom_level(&self) -> f32 {
        self.viewer_context.scale
    }

    /// Check if a PDF is loaded
    pub fn has_pdf(&self) -> bool {
        self.pdf_renderer.is_some()
    }

    /// Get the current file name
    pub fn current_file_name(&self) -> Option<String> {
        self.current_file
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
    }

    /// Update window size in viewer context
    pub fn update_window_size(&mut self, width: f32, height: f32) {
        self.viewer_context.window_size = Vector2F::new(width, height);
    }
}

impl Render for PdfViewerApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(rgb(0x1e1e1e))
            .text_color(rgb(0xcccccc))
            .child(self.render_toolbar(cx))
            .child(self.render_main_content(cx))
            .child(self.render_status_bar(cx))
    }
}

impl PdfViewerApp {
    /// Render the top toolbar
    fn render_toolbar(&self, _cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .h(px(48.0))
            .bg(rgb(0x2d2d30))
            .border_b_1()
            .border_color(rgb(0x3e3e42))
            .px_4()
            .child(
                // Left section - File operations
                div().flex().gap_2().child("Open PDF")
            )
            .child(
                // Middle section - Navigation
                div()
                    .flex()
                    .gap_2()
                    .items_center()
                    .children(
                        if self.has_pdf() {
                            Some(
                                div().child(
                                    format!(
                                        "Page {} / {}",
                                        self.current_page_display(),
                                        self.total_pages()
                                    )
                                )
                            )
                        } else {
                            None
                        }
                    )
            )
            .child(
                // Right section - Zoom controls
                div()
                    .flex()
                    .gap_2()
                    .children(
                        if self.has_pdf() {
                            Some(div().child(format!("{}%", (self.zoom_level() * 100.0) as i32)))
                        } else {
                            None
                        }
                    )
            )
    }

    /// Render the main content area
    fn render_main_content(&self, _cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .flex_1()
            .items_center()
            .justify_center()
            .w_full()
            .bg(rgb(0x252526))
            .child(
                if let Some(ref error) = self.error_message {
                    // Show error message
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_4()
                        .child(div().child("⚠ Error").text_xl().text_color(rgb(0xff6b6b)))
                        .child(div().child(error.clone()).text_color(rgb(0xcccccc)))
                } else if self.has_pdf() {
                    // Show PDF content
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .w_full()
                        .h_full()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap_2()
                                .child("PDF Rendering Area")
                                .text_color(rgb(0x808080))
                                .child(
                                    div()
                                        .child(
                                            format!(
                                                "Page {} of {} ({}% zoom)",
                                                self.current_page_display(),
                                                self.total_pages(),
                                                (self.zoom_level() * 100.0) as i32
                                            )
                                        )
                                        .text_sm()
                                )
                        )
                } else {
                    // Show welcome screen
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_4()
                        .child(div().child("📄 PDF Viewer").text_2xl().text_color(rgb(0xcccccc)))
                        .child(
                            div().child("Click 'Open PDF' to get started").text_color(rgb(0x808080))
                        )
                }
            )
    }

    /// Render the status bar
    fn render_status_bar(&self, _cx: &mut Context<Self>) -> Div {
        div()
            .flex()
            .items_center()
            .justify_between()
            .w_full()
            .h(px(24.0))
            .bg(rgb(0x007acc))
            .px_4()
            .child(
                div()
                    .child(self.current_file_name().unwrap_or_else(|| "No file loaded".to_string()))
                    .text_sm()
                    .text_color(rgb(0xffffff))
            )
            .child(div().child("Inkstone PDF Viewer").text_xs().text_color(rgb(0xe0e0e0)))
    }
}

/// Implementation of Interactive trait for PdfViewerApp
/// This allows the app to work with the viewer context system
impl Interactive for PdfViewerApp {
    type Event = ViewerEvent;
    type Backend = GpuiBackend;

    fn scene(&mut self, ctx: &mut ViewerContext<Self::Backend>) -> Scene {
        if let Some(ref mut renderer) = self.pdf_renderer {
            let page = ctx.page_nr();
            let transform = ctx.view_transform();

            // Render the page with the view transform from the context
            renderer.render_page(page, transform).unwrap_or_else(|_| Scene::new())
        } else {
            Scene::new()
        }
    }

    fn init(&mut self, _ctx: &mut ViewerContext<Self::Backend>, sender: Emitter<Self::Event>) {
        self.emitter = Some(sender);
    }

    fn title(&self) -> String {
        if let Some(ref name) = self.current_file_name() {
            format!("{} - Inkstone PDF Viewer", name)
        } else {
            "Inkstone PDF Viewer".to_string()
        }
    }

    fn window_size_hint(&self) -> Option<Vector2F> {
        Some(Vector2F::new(1200.0, 800.0))
    }

    fn event(&mut self, ctx: &mut ViewerContext<Self::Backend>, event: Self::Event) {
        match event {
            ViewerEvent::LoadFile(_path) => {
                // File loading will be handled externally through load_pdf
            }
            ViewerEvent::PageChanged(page) => {
                ctx.goto_page(page);
            }
            ViewerEvent::ZoomChanged(zoom) => {
                ctx.set_zoom(zoom);
            }
        }
    }

    fn cursor_moved(&mut self, _ctx: &mut ViewerContext<Self::Backend>, _pos: Vector2F) {
        // Can be implemented for hover effects or interactive features
    }
}
