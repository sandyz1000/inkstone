use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::label::Label;
use gpui_component::spinner::Spinner;
use gpui_component::*;
use rfd::AsyncFileDialog;
use std::path::PathBuf;
use std::sync::Arc;

use crate::renderer::PdfRenderer;

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
        log::info!("Loading PDF file: {:?}", path);
        match PdfRenderer::new(&path) {
            Ok(renderer) => {
                let num_pages = renderer.num_pages();

                self.current_file = Some(path);
                self.pdf_renderer = Some(renderer);
                self.error_message = None;
                self.current_page = 0;
                self.num_pages = num_pages;
                self.current_page_image = None;

                log::info!("✓ PDF loaded successfully with {} pages", num_pages);

                // Trigger async rendering
                self.render_current_page(cx);

                cx.notify();
            }
            Err(e) => {
                log::error!("Failed to load PDF: {}", e);
                self.error_message = Some(format!("Failed to load PDF: {}", e));
                self.pdf_renderer = None;
                self.current_page_image = None;
                cx.notify();
            }
        }
    }

    /// Render the current page synchronously
    fn render_current_page(&mut self, cx: &mut Context<Self>) {
        if let Some(renderer) = &mut self.pdf_renderer {
            let page_num = self.current_page;
            let zoom = self.zoom_level;

            log::info!("Rendering page {} with zoom {}...", page_num, zoom);

            let dpi = 150.0 * zoom;

            // Render the page (this might take time for complex PDFs)
            match renderer.render_page_to_image(page_num, dpi) {
                Ok(image) => {
                    let temp_path = std::env::temp_dir().join(format!(
                        "inkstone_page_{}_{}.png",
                        page_num,
                        (zoom * 100.0) as i32
                    ));

                    // Save the image
                    match image.save(&temp_path) {
                        Ok(_) => {
                            log::info!("✓ Page rendered to: {:?}", temp_path);
                            self.current_page_image = Some(temp_path.into());
                            self.error_message = None;
                        }
                        Err(e) => {
                            log::error!("Failed to save rendered page: {}", e);
                            self.error_message = Some(format!("Failed to save page: {}", e));
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to render page: {}", e);
                    self.error_message = Some(format!("Failed to render page: {}", e));
                }
            }

            cx.notify();
        }
    }

    /// Navigate to next page
    pub fn next_page(&mut self, cx: &mut Context<Self>) {
        if self.current_page + 1 < self.num_pages {
            self.current_page += 1;
            self.current_page_image = None;
            self.render_current_page(cx);
            cx.notify();
        }
    }

    /// Navigate to previous page
    pub fn prev_page(&mut self, cx: &mut Context<Self>) {
        if self.current_page > 0 {
            self.current_page -= 1;
            self.current_page_image = None;
            self.render_current_page(cx);
            cx.notify();
        }
    }

    /// Go to specific page (0-indexed)
    pub fn goto_page(&mut self, page: usize, cx: &mut Context<Self>) {
        if page < self.num_pages && page != self.current_page {
            self.current_page = page;
            self.current_page_image = None;
            self.render_current_page(cx);
            cx.notify();
        }
    }

    /// Zoom in
    pub fn zoom_in(&mut self, cx: &mut Context<Self>) {
        self.zoom_level *= 1.2;
        self.current_page_image = None;
        self.render_current_page(cx);
        cx.notify();
    }

    /// Zoom out
    pub fn zoom_out(&mut self, cx: &mut Context<Self>) {
        self.zoom_level /= 1.2;
        self.current_page_image = None;
        self.render_current_page(cx);
        cx.notify();
    }

    /// Reset zoom to 100%
    pub fn reset_zoom(&mut self, cx: &mut Context<Self>) {
        if self.zoom_level != 1.0 {
            self.zoom_level = 1.0;
            self.current_page_image = None;
            self.render_current_page(cx);
            cx.notify();
        }
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

        cx.spawn(async move |this, cx| {
            let file = AsyncFileDialog::new()
                .add_filter("PDF Files", &["pdf"])
                .set_title("Open PDF File")
                .pick_file()
                .await;

            if let Some(file) = file {
                let path = file.path().to_path_buf();
                log::info!("File selected: {:?}", path);

                _ = this.update(cx, |view, cx| {
                    view.load_pdf(path, cx);
                });
            } else {
                log::info!("No file selected");
            }
        })
        .detach();
    }
}

impl Render for PdfViewerApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle.clone();

        v_flex()
            .size_full()
            .bg(gpui::rgb(0x1e1e1e))
            .text_color(gpui::rgb(0xcccccc))
            .track_focus(&focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                log::debug!("Key pressed: {:?}", event.keystroke.key);

                if event.keystroke.modifiers.platform && event.keystroke.key == "o" {
                    this.open_file_dialog(cx);
                } else if event.keystroke.key == "ArrowRight" || event.keystroke.key == "right" {
                    this.next_page(cx);
                } else if event.keystroke.key == "ArrowLeft" || event.keystroke.key == "left" {
                    this.prev_page(cx);
                } else if event.keystroke.key == "=" || event.keystroke.key == "+" {
                    this.zoom_in(cx);
                } else if event.keystroke.key == "-" {
                    this.zoom_out(cx);
                } else if event.keystroke.key == "0" && event.keystroke.modifiers.platform {
                    this.reset_zoom(cx);
                }
            }))
            .child(self.render_toolbar(cx))
            .child(self.render_main_content(cx))
            .child(self.render_status_bar(cx))
    }
}

impl Focusable for PdfViewerApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl PdfViewerApp {
    /// Render the top toolbar
    fn render_toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .items_center()
            .justify_between()
            .w_full()
            .h(px(48.0))
            .bg(gpui::rgb(0x2d2d30))
            .border_b_1()
            .border_color(gpui::rgb(0x3e3e42))
            .px_4()
            .gap_4()
            .child(
                // Left section - File operations
                Button::new("open-pdf")
                    .primary()
                    .label("📁 Open PDF")
                    .on_click(cx.listener(|this, _, _, cx| {
                        log::info!("Open PDF button clicked");
                        this.open_file_dialog(cx);
                    })),
            )
            .child(
                // Middle section - Navigation
                h_flex()
                    .gap_2()
                    .items_center()
                    .when(self.has_pdf(), |this| {
                        this.child(
                            Button::new("prev-page")
                                .label("◀")
                                .disabled(self.current_page == 0)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.prev_page(cx);
                                })),
                        )
                        .child(Label::new(format!(
                            "Page {} / {}",
                            self.current_page_display(),
                            self.total_pages()
                        )))
                        .child(
                            Button::new("next-page")
                                .label("▶")
                                .disabled(self.current_page + 1 >= self.num_pages)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.next_page(cx);
                                })),
                        )
                    }),
            )
            .child(
                // Right section - Zoom controls
                h_flex()
                    .gap_2()
                    .items_center()
                    .when(self.has_pdf(), |this| {
                        this.child(Button::new("zoom-out").label("−").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.zoom_out(cx);
                            },
                        )))
                        .child(Label::new(format!(
                            "{}%",
                            (self.zoom_level() * 100.0) as i32
                        )))
                        .child(Button::new("zoom-in").label("+").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.zoom_in(cx);
                            },
                        )))
                        .child(
                            Button::new("zoom-reset")
                                .label("100%")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.reset_zoom(cx);
                                })),
                        )
                    }),
            )
    }

    /// Render the main content area
    fn render_main_content(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .flex_1()
            .items_center()
            .justify_center()
            .w_full()
            .bg(gpui::rgb(0x252526))
            .child(if let Some(ref error) = self.error_message {
                // Show error message
                v_flex()
                    .items_center()
                    .gap_4()
                    .p_8()
                    .child(
                        div()
                            .child("⚠ Error")
                            .text_2xl()
                            .text_color(gpui::rgb(0xff6b6b)),
                    )
                    .child(div().child(error.clone()).text_color(gpui::rgb(0xcccccc)))
            } else if let Some(ref image_path) = self.current_page_image {
                // Display the rendered PDF page image
                v_flex()
                    .items_center()
                    .justify_center()
                    .p_4()
                    .gap_4()
                    .child(
                        // The actual PDF page image
                        img(image_path.clone()).max_w(px(1000.0)).max_h(px(800.0)),
                    )
                    .child(
                        // Page info overlay
                        div()
                            .child(format!(
                                "Page {} of {} ({}% zoom)",
                                self.current_page_display(),
                                self.total_pages(),
                                (self.zoom_level() * 100.0) as i32
                            ))
                            .text_sm()
                            .text_color(gpui::rgb(0x808080)),
                    )
            } else if self.has_pdf() {
                // PDF loaded but image not yet rendered
                v_flex()
                    .items_center()
                    .gap_4()
                    .p_8()
                    .child(
                        div()
                            .child("📄 Rendering PDF page...")
                            .text_2xl()
                            .text_color(gpui::rgb(0x4CAF50)),
                    )
                    .child(Spinner::new())
            } else {
                // Show welcome screen
                v_flex()
                    .items_center()
                    .gap_4()
                    .p_12()
                    .child(
                        div()
                            .child("📄 Inkstone PDF Viewer")
                            .text_3xl()
                            .text_color(gpui::rgb(0xcccccc)),
                    )
                    .child(
                        div()
                            .child("Click 'Open PDF' or press Cmd+O to load a document")
                            .text_color(gpui::rgb(0x808080)),
                    )
                    .child(
                        v_flex()
                            .gap_2()
                            .mt_6()
                            .child(
                                Label::new("Keyboard shortcuts:").text_color(gpui::rgb(0x808080)),
                            )
                            .child(
                                Label::new("• Cmd+O - Open PDF")
                                    .text_sm()
                                    .text_color(gpui::rgb(0x606060)),
                            )
                            .child(
                                Label::new("• ← → - Previous/Next page")
                                    .text_sm()
                                    .text_color(gpui::rgb(0x606060)),
                            )
                            .child(
                                Label::new("• + - - Zoom in/out")
                                    .text_sm()
                                    .text_color(gpui::rgb(0x606060)),
                            )
                            .child(
                                Label::new("• Cmd+0 - Reset zoom")
                                    .text_sm()
                                    .text_color(gpui::rgb(0x606060)),
                            ),
                    )
            })
    }

    /// Render the status bar
    fn render_status_bar(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .items_center()
            .justify_between()
            .w_full()
            .h(px(24.0))
            .bg(gpui::rgb(0x007acc))
            .px_4()
            .child(
                div()
                    .child(
                        self.current_file_name()
                            .unwrap_or_else(|| "No file loaded".to_string()),
                    )
                    .text_sm()
                    .text_color(gpui::rgb(0xffffff)),
            )
            .child(
                div()
                    .child("Inkstone PDF Viewer - Built with GPUI")
                    .text_xs()
                    .text_color(gpui::rgb(0xe0e0e0)),
            )
    }
}
