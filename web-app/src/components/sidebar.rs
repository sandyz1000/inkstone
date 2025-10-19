use dioxus::prelude::*;
use crate::app::AppState;

#[derive(Clone, Copy, PartialEq)]
enum SidebarTab {
    Thumbnails,
    Bookmarks,
    Attachments,
}

#[component]
pub fn Sidebar(app_state: Signal<AppState>) -> Element {
    let mut active_tab = use_signal(|| SidebarTab::Thumbnails);

    rsx! {
        div {
            class: "sidebar",
            style: "width: 250px; background: #252526; border-right: 1px solid #333; display: flex; flex-direction: column;",
            
            // Tab headers
            div {
                class: "sidebar-tabs",
                style: "display: flex; border-bottom: 1px solid #333;",
                
                button {
                    onclick: move |_| active_tab.set(SidebarTab::Thumbnails),
                    style: format!(
                        "flex: 1; padding: 12px; border-bottom: 2px solid {}; background: {};",
                        if matches!(*active_tab.read(), SidebarTab::Thumbnails) { "#667eea" } else { "transparent" },
                        if matches!(*active_tab.read(), SidebarTab::Thumbnails) { "#2d2d2d" } else { "transparent" }
                    ),
                    "📑 Pages"
                }
                
                button {
                    onclick: move |_| active_tab.set(SidebarTab::Bookmarks),
                    style: format!(
                        "flex: 1; padding: 12px; border-bottom: 2px solid {}; background: {};",
                        if matches!(*active_tab.read(), SidebarTab::Bookmarks) { "#667eea" } else { "transparent" },
                        if matches!(*active_tab.read(), SidebarTab::Bookmarks) { "#2d2d2d" } else { "transparent" }
                    ),
                    "🔖 Bookmarks"
                }
                
                button {
                    onclick: move |_| active_tab.set(SidebarTab::Attachments),
                    style: format!(
                        "flex: 1; padding: 12px; border-bottom: 2px solid {}; background: {};",
                        if matches!(*active_tab.read(), SidebarTab::Attachments) { "#667eea" } else { "transparent" },
                        if matches!(*active_tab.read(), SidebarTab::Attachments) { "#2d2d2d" } else { "transparent" }
                    ),
                    "📎 Files"
                }
            }
            
            // Tab content
            div {
                class: "sidebar-content",
                style: "flex: 1; overflow-y: auto; padding: 8px;",
                
                match *active_tab.read() {
                    SidebarTab::Thumbnails => rsx! { ThumbnailsView { app_state: app_state } },
                    SidebarTab::Bookmarks => rsx! { BookmarksView {} },
                    SidebarTab::Attachments => rsx! { AttachmentsView {} },
                }
            }
        }
    }
}

#[component]
fn ThumbnailsView(app_state: Signal<AppState>) -> Element {
    let state = app_state.read();
    let total_pages = state.total_pages;
    let current_page = state.current_page;

    rsx! {
        div {
            class: "thumbnails",
            style: "display: flex; flex-direction: column; gap: 12px;",
            
            if total_pages == 0 {
                div {
                    style: "text-align: center; color: #666; padding: 24px;",
                    "No pages to display"
                }
            } else {
                for page_num in 1..=total_pages {
                    div {
                        key: "{page_num}",
                        onclick: move |_| {
                            app_state.write().current_page = page_num;
                            log::info!("Navigate to page {}", page_num);
                        },
                        style: format!(
                            "padding: 8px; border-radius: 4px; cursor: pointer; background: {}; border: 2px solid {};",
                            if page_num == current_page { "#2d2d2d" } else { "transparent" },
                            if page_num == current_page { "#667eea" } else { "transparent" }
                        ),
                        
                        div {
                            style: "width: 100%; aspect-ratio: 8.5/11; background: white; border-radius: 2px; display: flex; align-items: center; justify-content: center; color: #999; font-size: 12px;",
                            "Page {page_num}"
                        }
                        
                        div {
                            style: "text-align: center; margin-top: 4px; font-size: 12px; color: #999;",
                            "Page {page_num}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn BookmarksView() -> Element {
    rsx! {
        div {
            style: "text-align: center; color: #666; padding: 24px;",
            "No bookmarks"
        }
    }
}

#[component]
fn AttachmentsView() -> Element {
    rsx! {
        div {
            style: "text-align: center; color: #666; padding: 24px;",
            "No attachments"
        }
    }
}
