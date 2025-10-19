use dioxus::prelude::*;
use crate::app::{ AppState, ViewMode };

#[component]
pub fn Toolbar(app_state: Signal<AppState>) -> Element {
    let state = app_state.read();
    let current_page = state.current_page;
    let total_pages = state.total_pages;
    let zoom_level = state.zoom_level;

    rsx! {
        div {
            class: "toolbar",
            style: "display: flex; align-items: center; justify-content: space-between; padding: 8px 16px; background: #2d2d2d; border-bottom: 1px solid #444; min-height: 48px;",
            
            // Navigation controls
            div {
                class: "nav-controls",
                style: "display: flex; align-items: center; gap: 8px;",
                
                button {
                    onclick: move |_| {
                        if app_state.read().current_page > 1 {
                            app_state.write().current_page -= 1;
                            log::info!("Previous page: {}", app_state.read().current_page);
                        }
                    },
                    disabled: current_page <= 1,
                    title: "Previous Page",
                    "◀"
                }
                
                div {
                    style: "display: flex; align-items: center; gap: 4px; padding: 0 8px;",
                    
                    input {
                        r#type: "number",
                        value: "{current_page}",
                        min: "1",
                        max: "{total_pages}",
                        oninput: move |evt| {
                            if let Ok(page) = evt.value().parse::<usize>() {
                                if page >= 1 && page <= total_pages {
                                    app_state.write().current_page = page;
                                }
                            }
                        },
                    }
                    
                    span { 
                        style: "color: #999;",
                        "/ {total_pages}" 
                    }
                }
                
                button {
                    onclick: move |_| {
                        if app_state.read().current_page < total_pages {
                            app_state.write().current_page += 1;
                            log::info!("Next page: {}", app_state.read().current_page);
                        }
                    },
                    disabled: current_page >= total_pages || total_pages == 0,
                    title: "Next Page",
                    "▶"
                }
            }
            
            // Zoom controls
            div {
                class: "zoom-controls",
                style: "display: flex; align-items: center; gap: 8px;",
                
                button {
                    onclick: move |_| {
                        let new_zoom = (app_state.read().zoom_level - 0.1).max(0.1);
                        app_state.write().zoom_level = new_zoom;
                        log::info!("Zoom out: {:.1}%", new_zoom * 100.0);
                    },
                    title: "Zoom Out",
                    "🔍−"
                }
                
                button {
                    onclick: move |_| {
                        app_state.write().zoom_level = 1.0;
                        log::info!("Reset zoom: 100%");
                    },
                    title: "Reset Zoom",
                    "{(zoom_level * 100.0) as i32}%"
                }
                
                button {
                    onclick: move |_| {
                        let new_zoom = (app_state.read().zoom_level + 0.1).min(5.0);
                        app_state.write().zoom_level = new_zoom;
                        log::info!("Zoom in: {:.1}%", new_zoom * 100.0);
                    },
                    title: "Zoom In",
                    "🔍+"
                }
                
                button {
                    onclick: move |_| {
                        // Fit to page width
                        app_state.write().zoom_level = 1.2; // TODO: Calculate actual fit
                        log::info!("Fit to width");
                    },
                    title: "Fit to Width",
                    "↔️"
                }
                
                button {
                    onclick: move |_| {
                        // Fit to page
                        app_state.write().zoom_level = 1.0; // TODO: Calculate actual fit
                        log::info!("Fit to page");
                    },
                    title: "Fit to Page",
                    "⛶"
                }
            }
            
            // View mode controls
            div {
                class: "view-controls",
                style: "display: flex; align-items: center; gap: 8px;",
                
                button {
                    onclick: move |_| {
                        app_state.write().view_mode = ViewMode::SinglePage;
                        log::info!("View mode: Single Page");
                    },
                    title: "Single Page",
                    style: if matches!(state.view_mode, ViewMode::SinglePage) { 
                        "background: rgba(255, 255, 255, 0.2);" 
                    } else { "" },
                    "📄"
                }
                
                button {
                    onclick: move |_| {
                        app_state.write().view_mode = ViewMode::ContinuousScroll;
                        log::info!("View mode: Continuous Scroll");
                    },
                    title: "Continuous Scroll",
                    style: if matches!(state.view_mode, ViewMode::ContinuousScroll) { 
                        "background: rgba(255, 255, 255, 0.2);" 
                    } else { "" },
                    "📜"
                }
                
                button {
                    onclick: move |_| {
                        app_state.write().view_mode = ViewMode::TwoPage;
                        log::info!("View mode: Two Page");
                    },
                    title: "Two Page",
                    style: if matches!(state.view_mode, ViewMode::TwoPage) { 
                        "background: rgba(255, 255, 255, 0.2);" 
                    } else { "" },
                    "📖"
                }
            }
        }
    }
}
