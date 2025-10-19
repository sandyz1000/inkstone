use dioxus::prelude::*;
use crate::app::AppState;

#[component]
pub fn Header(app_state: Signal<AppState>) -> Element {
    let file_name = app_state
        .read()
        .file_name.clone()
        .unwrap_or_else(|| "Inkstone PDF Viewer".to_string());

    rsx! {
        header {
            class: "app-header",
            style: "display: flex; align-items: center; justify-content: space-between; padding: 12px 24px; background: #252526; border-bottom: 1px solid #333; min-height: 50px;",
            
            div {
                class: "header-left",
                style: "display: flex; align-items: center; gap: 16px;",
                
                button {
                    class: "menu-button",
                    onclick: move |_| {
                        let current_visibility = app_state.read().sidebar_visible;
                        app_state.write().sidebar_visible = !current_visibility;
                    },
                    title: "Toggle Sidebar",
                    style: "font-size: 20px;",
                    "☰"
                }
                
                div {
                    class: "logo",
                    style: "display: flex; align-items: center; gap: 8px;",
                    
                    div {
                        style: "width: 32px; height: 32px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); border-radius: 6px; display: flex; align-items: center; justify-content: center; font-weight: bold; color: white;",
                        "IS"
                    }
                    
                    h1 {
                        style: "font-size: 18px; font-weight: 600;",
                        "{file_name}"
                    }
                }
            }
            
            div {
                class: "header-right",
                style: "display: flex; align-items: center; gap: 12px;",
                
                button {
                    class: "file-button",
                    onclick: move |_| {
                        log::info!("Open file dialog");
                        // TODO: Implement file opening
                    },
                    title: "Open File",
                    "📂 Open"
                }
                
                button {
                    class: "download-button",
                    onclick: move |_| {
                        log::info!("Download PDF");
                        // TODO: Implement download
                    },
                    title: "Download",
                    "⬇️ Download"
                }
                
                button {
                    class: "print-button",
                    onclick: move |_| {
                        log::info!("Print PDF");
                        // TODO: Implement printing
                    },
                    title: "Print",
                    "🖨️ Print"
                }
            }
        }
    }
}
