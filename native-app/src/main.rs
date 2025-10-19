use gpui::*;
use native_app::PdfViewerApp;
use std::path::PathBuf;

fn main() {
    env_logger::init();

    // Create and run the GPUI application
    Application::new().run(|cx: &mut App| {
        // Get command line arguments for PDF path
        let args: Vec<String> = std::env::args().collect();
        let pdf_path = if args.len() > 1 { Some(PathBuf::from(&args[1])) } else { None };

        // Create the PDF viewer entity first
        let pdf_viewer = cx.new(|cx| {
            let mut app = PdfViewerApp::new(cx);

            // Load PDF if path was provided
            if let Some(path) = pdf_path {
                app.load_pdf(path, cx);
            }

            app
        });

        // Open main window with the PDF viewer
        cx.open_window(
            WindowOptions {
                window_bounds: Some(
                    WindowBounds::Windowed(Bounds::centered(None, size(px(1200.0), px(800.0)), cx))
                ),
                titlebar: Some(TitlebarOptions {
                    title: Some("Inkstone PDF Viewer".into()),
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                ..Default::default()
            },
            move |_window, _cx| {
                // Return the PDF viewer entity
                pdf_viewer
            }
        ).unwrap();
    });
}
