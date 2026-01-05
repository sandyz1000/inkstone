use gpui::*;
use gpui_component::Root;

mod app;
mod renderer;

use app::PdfViewerApp;

fn main() {
    // Setup fonts path
    if let Ok(current_dir) = std::env::current_dir() {
        let fonts_dir = current_dir.join("fonts");
        if fonts_dir.exists() {
            unsafe {
                std::env::set_var("STANDARD_FONTS", fonts_dir.display().to_string());
            }
            log::info!("✓ STANDARD_FONTS set to: {}", fonts_dir.display());
        }
    }
    
    env_logger::init();
    
    let app = Application::new();

    app.run(move |cx| {
        // Initialize GPUI Component
        gpui_component::init(cx);

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: Point::new(px(100.0), px(100.0)),
                        size: Size {
                            width: px(1200.0),
                            height: px(800.0),
                        },
                    })),
                    titlebar: Some(TitlebarOptions {
                        title: Some("Inkstone PDF Viewer".into()),
                        appears_transparent: false,
                        traffic_light_position: None,
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|cx| PdfViewerApp::new(cx));
                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
            