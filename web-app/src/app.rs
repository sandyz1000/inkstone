use dioxus::prelude::*;
use crate::components::{ Header, Toolbar, PDFCanvas, Sidebar };

#[derive(Clone, Copy, PartialEq)]
pub enum ViewMode {
    SinglePage,
    ContinuousScroll,
    TwoPage,
}

#[derive(Clone)]
pub struct AppState {
    pub current_page: usize,
    pub total_pages: usize,
    pub zoom_level: f32,
    pub view_mode: ViewMode,
    pub sidebar_visible: bool,
    pub file_loaded: bool,
    pub file_name: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_page: 1,
            total_pages: 0,
            zoom_level: 1.0,
            view_mode: ViewMode::SinglePage,
            sidebar_visible: false,
            file_loaded: false,
            file_name: None,
        }
    }
}

#[component]
pub fn App() -> Element {
    let mut app_state = use_signal(AppState::default);

    rsx! {
        div {
            class: "app-container",
            style: "display: flex; flex-direction: column; height: 100vh; width: 100vw; overflow: hidden; background: #1e1e1e;",
            
            // Header with title and menu
            Header {
                app_state: app_state
            }
            
            div {
                class: "main-content",
                style: "display: flex; flex: 1; overflow: hidden;",
                
                // Sidebar (thumbnails, bookmarks, etc.)
                if app_state.read().sidebar_visible {
                    Sidebar {
                        app_state: app_state
                    }
                }
                
                div {
                    class: "viewer-area",
                    style: "display: flex; flex-direction: column; flex: 1; overflow: hidden;",
                    
                    // Toolbar with navigation and zoom controls
                    Toolbar {
                        app_state: app_state
                    }
                    
                    // Main PDF viewing area with WebGL canvas
                    div {
                        class: "canvas-container",
                        style: "flex: 1; display: flex; justify-content: center; align-items: center; overflow: auto; background: #2d2d2d;",
                        
                        PDFCanvas {
                            app_state: app_state
                        }
                    }
                }
            }
        }
        
        // Global styles
        style { {GLOBAL_STYLES} }
    }
}

const GLOBAL_STYLES: &str =
    r#"
    * {
        margin: 0;
        padding: 0;
        box-sizing: border-box;
    }
    
    body {
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
        color: #e0e0e0;
        background: #1e1e1e;
    }
    
    button {
        cursor: pointer;
        border: none;
        background: none;
        color: inherit;
        font-size: inherit;
        padding: 8px 16px;
        border-radius: 4px;
        transition: background-color 0.2s;
    }
    
    button:hover {
        background: rgba(255, 255, 255, 0.1);
    }
    
    button:active {
        background: rgba(255, 255, 255, 0.2);
    }
    
    input[type="number"] {
        background: #2d2d2d;
        border: 1px solid #444;
        color: #e0e0e0;
        padding: 4px 8px;
        border-radius: 4px;
        width: 60px;
        text-align: center;
    }
    
    canvas {
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
    }
    
    ::-webkit-scrollbar {
        width: 12px;
        height: 12px;
    }
    
    ::-webkit-scrollbar-track {
        background: #1e1e1e;
    }
    
    ::-webkit-scrollbar-thumb {
        background: #444;
        border-radius: 6px;
    }
    
    ::-webkit-scrollbar-thumb:hover {
        background: #555;
    }
"#;
