use gpui::*;
use pathfinder_geometry::vector::Vector2F;
use std::path::PathBuf;
use std::sync::Arc;
use rfd::FileDialog;

use crate::renderer::PdfRenderer;

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
pub struct PdfViewerApp {
    /// Currently loaded PDF file path
    current_file: Option<PathBuf>,
    /// PDF renderer instance
    pdf_renderer: Option<PdfRenderer>,
    /// Current page number (0-indexed)
    current_page: usize,
    /// Total number of pages
    num_pages: usize,
    /// Current zoom level (1.0 = 100%)
    zoom_level: f32,
    /// Error message if any
    error_message: Option<String>,
    /// Focus handle for keyboard events
    focus_handle: FocusHandle,
    /// Cached rendered page image path
    current_page_image: Option<Arc<std::path::Path>>,
}

impl PdfViewerApp {
    /// Create a new PDF Viewer application
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            current_file: None,
            pdf_renderer: None,
            current_page: 0,
            num_pages: 0,
            zoom_level: 1.0,
            error_message: None,
            focus_handle: cx.focus_handle(),
            current_page_image: None,
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
                self.current_page = 0;
                self.num_pages = num_pages;
                self.current_page_image = None; // Don't render yet
                
                // Trigger async rendering
                self.render_current_page(cx);

                cx.notify();
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to load PDF: {}", e));
                self.pdf_renderer = None;
                self.current_page_image = None;
                cx.notify();
            }
        }
    }
    
    /// Render the current page asynchronously
    fn render_current_page(&mut self, cx: &mut Context<Self>) {
        if let Some(renderer) = &mut self.pdf_renderer {
            log::info!("Rendering page {} to image...", self.current_page);
            
            // Render directly (CGL context will be created on this thread)
            match renderer.render_page_to_image(self.current_page, 150.0) {
                Ok(image) => {
                    // Save to temp directory
                    let temp_path = std::env::temp_dir().join(format!("inkstone_page_{}.png", self.current_page));
                    match image.save(&temp_path) {
                        Ok(_) => {
                            log::info!("✓ Successfully rendered page {} to: {:?}", self.current_page, temp_path);
                            self.current_page_image = Some(temp_path.into());
                            cx.notify();
                        }
                        Err(e) => {
                            log::warn!("Failed to save rendered page: {}", e);
                            self.error_message = Some(format!("Failed to save page: {}", e));
                            cx.notify();
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to render page to image: {}", e);
                    self.error_message = Some(format!("Failed to render page: {}", e));
                    cx.notify();
                }
            }
        }
    }

    /// Navigate to next page
    pub fn next_page(&mut self, cx: &mut Context<Self>) {
        if self.current_page + 1 < self.num_pages {
            self.current_page += 1;
            self.current_page_image = None; // Clear old image
            self.render_current_page(cx);
            cx.notify();
        }
    }

    /// Navigate to previous page
    pub fn prev_page(&mut self, cx: &mut Context<Self>) {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.current_page_image = None; // Clear old image
            self.render_current_page(cx);
            cx.notify();
        }
    }

    /// Go to specific page (0-indexed)
    pub fn goto_page(&mut self, page: usize, cx: &mut Context<Self>) {
        if page < self.num_pages && page != self.current_page {
            self.current_page = page;
            self.current_page_image = None; // Clear old image
            self.render_current_page(cx);
            cx.notify();
        }
    }

    /// Zoom in
    pub fn zoom_in(&mut self, cx: &mut Context<Self>) {
        self.zoom_level *= 1.2;
        cx.notify();
    }

    /// Zoom out
    pub fn zoom_out(&mut self, cx: &mut Context<Self>) {
        self.zoom_level /= 1.2;
        cx.notify();
    }

    /// Reset zoom to 100%
    pub fn reset_zoom(&mut self, cx: &mut Context<Self>) {
        self.zoom_level = 1.0;
        cx.notify();
    }

    /// Get current page number (1-indexed for display)
    pub fn current_page_display(&self) -> usize {
        self.current_page + 1
    }

    /// Get total pages
    pub fn total_pages(&self) -> usize {
        self.num_pages
    }

    /// Get current zoom level
    pub fn zoom_level(&self) -> f32 {
        self.zoom_level
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

    /// Open file dialog and load selected PDF
    pub fn open_file_dialog(&mut self, cx: &mut Context<Self>) {
        log::info!("Opening file dialog...");
        match FileDialog::new()
            .add_filter("PDF Files", &["pdf"])
            .set_title("Open PDF File")
            .pick_file()
        {
            Some(file) => {
                log::info!("File selected: {:?}", file);
                self.load_pdf(file, cx);
            }
            None => {
                log::info!("No file selected");
            }
        }
    }
}

impl Render for PdfViewerApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle.clone();
        
        div()
            .flex()
            .flex_col()
            .w_full()
            .h_full()
            .bg(rgb(0x1e1e1e))
            .text_color(rgb(0xcccccc))
            .track_focus(&focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                // Handle keyboard shortcuts
                log::info!("Key pressed: {:?}, modifiers: {:?}", event.keystroke.key, event.keystroke.modifiers);
                
                if event.keystroke.modifiers.platform && event.keystroke.key == "o" {
                    log::info!("Cmd+O pressed - opening file dialog");
                    this.open_file_dialog(cx);
                } else if event.keystroke.key == "ArrowRight" {
                    log::info!("Arrow Right pressed");
                    this.next_page(cx);
                } else if event.keystroke.key == "ArrowLeft" {
                    log::info!("Arrow Left pressed");
                    this.prev_page(cx);
                } else if event.keystroke.key == "=" || event.keystroke.key == "+" {
                    log::info!("Zoom in");
                    this.zoom_in(cx);
                } else if event.keystroke.key == "-" {
                    log::info!("Zoom out");
                    this.zoom_out(cx);
                }
            }))
            .child(self.render_toolbar(cx))
            .child(self.render_main_content(cx))
            .child(self.render_status_bar(cx))
    }
}

impl Focusable for PdfViewerApp {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl PdfViewerApp {
    /// Render the top toolbar
    fn render_toolbar(&self, cx: &mut Context<Self>) -> Div {
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
                div()
                    .flex()
                    .gap_2()
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .bg(rgb(0x0e639c))
                            .rounded_md()
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0x1177bb)))
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _event: &MouseDownEvent, _window, cx| {
                                log::info!("Open PDF button clicked!");
                                this.open_file_dialog(cx);
                            }))
                            .child("📁 Open PDF")
                    )
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
                                div()
                                    .flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        div()
                                            .px_2()
                                            .py_1()
                                            .bg(rgb(0x3e3e42))
                                            .rounded_md()
                                            .child("◀ (←)")
                                    )
                                    .child(
                                        div().child(
                                            format!(
                                                "Page {} / {}",
                                                self.current_page_display(),
                                                self.total_pages()
                                            )
                                        )
                                    )
                                    .child(
                                        div()
                                            .px_2()
                                            .py_1()
                                            .bg(rgb(0x3e3e42))
                                            .rounded_md()
                                            .child("▶ (→)")
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
                            Some(
                                div()
                                    .flex()
                                    .gap_2()
                                    .items_center()
                                    .child(
                                        div()
                                            .px_2()
                                            .py_1()
                                            .bg(rgb(0x3e3e42))
                                            .rounded_md()
                                            .child("− (−)")
                                    )
                                    .child(
                                        div().child(format!("{}%", (self.zoom_level() * 100.0) as i32))
                                    )
                                    .child(
                                        div()
                                            .px_2()
                                            .py_1()
                                            .bg(rgb(0x3e3e42))
                                            .rounded_md()
                                            .child("+ (+)")
                                    )
                            )
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
                } else if let Some(ref image_path) = self.current_page_image {
                    // Display the rendered PDF page image
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap_4()
                        .child(
                            // The actual PDF page image
                            img(image_path.clone())
                                .w_full()
                                .max_w(px(800.0))
                        )
                        .child(
                            // Page info overlay
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
                                .text_color(rgb(0xcccccc))
                        )
                } else if self.has_pdf() {
                    // PDF loaded but image not yet rendered
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_4()
                        .child(
                            div()
                                .child("📄 Rendering PDF...")
                                .text_2xl()
                                .text_color(rgb(0x4CAF50))
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
