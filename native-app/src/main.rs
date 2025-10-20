use iced::widget::{button, column, container, image, row, text, horizontal_space, vertical_space, scrollable};
use iced::{Alignment, Element, Length, Task, Theme, Color};
use std::path::PathBuf;

mod renderer;
use renderer::PdfRenderer;

fn main() -> iced::Result {
    if let Ok(current_dir) = std::env::current_dir() {
        let fonts_dir = current_dir.join("fonts");
        if fonts_dir.exists() {
            unsafe {
                std::env::set_var("STANDARD_FONTS", fonts_dir.display().to_string());
            }
            println!("✓ STANDARD_FONTS set to: {}", fonts_dir.display());
        }
    }
    env_logger::init();
    iced::application("Inkstone PDF Viewer", PdfViewerApp::update, PdfViewerApp::view)
        .theme(|_| Theme::Dark)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    OpenFile,
    FileOpened(Result<PathBuf, String>),
    NextPage,
    PrevPage,
    ZoomIn,
    ZoomOut,
}

struct PdfViewerApp {
    current_file: Option<PathBuf>,
    pdf_renderer: Option<PdfRenderer>,
    current_page: usize,
    num_pages: usize,
    zoom_level: f32,
    error_message: Option<String>,
    rendered_image: Option<image::Handle>,
}

impl Default for PdfViewerApp {
    fn default() -> Self {
        Self {
            current_file: None,
            pdf_renderer: None,
            current_page: 0,
            num_pages: 0,
            zoom_level: 1.0,
            error_message: None,
            rendered_image: None,
        }
    }
}

impl PdfViewerApp {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenFile => Task::perform(
                async {
                    if let Some(file) = rfd::AsyncFileDialog::new()
                        .add_filter("PDF Files", &["pdf"])
                        .pick_file()
                        .await
                    {
                        let path = file.path().to_path_buf();
                        Ok(path)
                    } else {
                        Err("No file selected".to_string())
                    }
                },
                Message::FileOpened
            ),
            Message::FileOpened(Ok(path)) => {
                // Clear the old state first
                self.rendered_image = None;
                self.error_message = None;
                self.pdf_renderer = None;
                self.current_page = 0;
                self.num_pages = 0;
                self.zoom_level = 1.0;
                
                match PdfRenderer::new(&path) {
                    Ok(renderer) => {
                        self.num_pages = renderer.num_pages();
                        self.pdf_renderer = Some(renderer);
                        self.current_file = Some(path);
                        self.render_current_page();
                    }
                    Err(e) => {
                        self.error_message = Some(e);
                    }
                }
                Task::none()
            }
            Message::FileOpened(Err(e)) => {
                self.error_message = Some(e);
                Task::none()
            }
            Message::NextPage => {
                if self.current_page + 1 < self.num_pages {
                    self.current_page += 1;
                    self.render_current_page();
                }
                Task::none()
            }
            Message::PrevPage => {
                if self.current_page > 0 {
                    self.current_page -= 1;
                    self.render_current_page();
                }
                Task::none()
            }
            Message::ZoomIn => {
                self.zoom_level *= 1.2;
                self.render_current_page();
                Task::none()
            }
            Message::ZoomOut => {
                self.zoom_level /= 1.2;
                self.render_current_page();
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<Message> {
        column![
            self.toolbar(),
            self.main_content(),
        ]
        .into()
    }

    fn render_current_page(&mut self) {
        if let Some(renderer) = &mut self.pdf_renderer {
            let dpi = 150.0 * self.zoom_level;
            match renderer.render_page_to_image(self.current_page, dpi) {
                Ok(img) => {
                    let temp_path = std::env::temp_dir().join(format!("inkstone_page_{}.png", self.current_page));
                    match img.save(&temp_path) {
                        Ok(_) => {
                            self.rendered_image = Some(image::Handle::from_path(&temp_path));
                            self.error_message = None;
                            println!("✓ Rendered page {} to: {}", self.current_page + 1, temp_path.display());
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Failed to save image: {}", e));
                            println!("✗ Failed to save image: {}", e);
                        }
                    }
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to render page: {}", e));
                    println!("✗ Failed to render page: {}", e);
                }
            }
        }
    }

    fn toolbar(&self) -> Element<Message> {
        row![
            button("Open PDF").on_press(Message::OpenFile),
            text(format!("Page {}/{}", self.current_page + 1, self.num_pages)),
            button("Prev").on_press_maybe(if self.current_page > 0 { Some(Message::PrevPage) } else { None }),
            button("Next").on_press_maybe(if self.current_page + 1 < self.num_pages { Some(Message::NextPage) } else { None }),
            button("Zoom+").on_press(Message::ZoomIn),
            button("Zoom-").on_press(Message::ZoomOut),
        ]
        .spacing(10)
        .padding(10)
        .into()
    }

    fn main_content(&self) -> Element<Message> {
        if let Some(ref error) = self.error_message {
            column![
                text("Error:").size(20),
                text(error).size(14)
            ]
            .padding(20)
            .into()
        } else if let Some(ref img_handle) = self.rendered_image {
            scrollable(image(img_handle.clone())).into()
        } else if self.pdf_renderer.is_some() {
            text("Rendering page...").into()
        } else {
            column![
                text("Inkstone PDF Viewer").size(30),
                text("Click 'Open PDF' to load a document").size(14)
            ]
            .padding(40)
            .spacing(10)
            .into()
        }
    }
}
