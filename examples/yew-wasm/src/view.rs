#![allow(unused)]
use std::rc::Rc;
use stylist::css;
use yew::prelude::*;
use yew_hooks::prelude::*;
use yew_hooks::use_async;

use super::{
    error::ApiError, pages::PdfPages, progressbar::ProgressBar, searchbar::SearchBar,
    thumbnail::ThumbBar, toolbox::Toolbox,
    types::{ PDFFindController, PDFLinkService, PDFViewer }
};

const ZOOM_STEP: f64 = 0.2;

#[derive(Properties, Clone, PartialEq, Eq)]
struct PdfViewerProps {
    pub url: String,
    pub show_progress_bar: bool,
    pub show_toolbox: bool,
    pub show_thumbnail_sidebar: bool,
}


#[derive(Debug, Properties, PartialEq)]
pub struct GlobalViweProps {
    children: Children,
}

// TODO: Wrap duplicate code here
fn wrap_async<T: PartialEq + std::fmt::Debug + Clone>(state: T, closure: impl Fn()) -> T {
    todo!()
}

#[function_component]
pub fn PdfViewerComponent(
    PdfViewerProps {
        url,
        show_progress_bar,
        show_toolbox,
        show_thumbnail_sidebar,
    }: &PdfViewerProps,
) -> Html {
    let doc: UseStateHandle<Option<PdfDocument>> = use_state(|| None );
    let viewer: UseStateHandle<Option<PDFViewer>> = use_state(|| None);
    let scale = use_state(|| 0.0 as f64);
    let progress = use_state(|| 0.0 as f64);
    let current_page = use_state(|| 0 as usize);
    let show_search_bar  = use_state(|| false);
    let show_thumb_sidebar = use_state(|| false);

    let scroll_to = {
        let current_page = current_page.clone();
        Callback::from( move |c: usize| current_page.set(c) )
    };

    let set_pdf = {
        let doc = doc.clone();
        Callback::from( move |pdf: PdfDocument| doc.set(Some(pdf)) )
    };

    let set_viewer = {
        let viewer = viewer.clone();
        Callback::from(move |view: PDFViewer| {
            viewer.set(Some(view));
        })
    };

    // Define the `FindController` here
    let find_controller = use_state(|| None::<PDFFindController>);

    let set_find_controller = {
        let find_controller = find_controller.clone();
        Callback::from(move |controller: PDFFindController| {
            find_controller.set(Some(controller));
        })
    };

    let toggle_search_bar = {
        let show_search_bar = show_search_bar.clone();
        Callback::from(move |e| show_search_bar.set(!(*show_search_bar)) )
    };

    let set_progress = {
        let progress = progress.clone();
        Callback::from(move |c: f64| progress.set(c))
    };

    let on_change_page = {
        let scroll_to = scroll_to.clone();
        Callback::from(move |c: usize| {
            scroll_to.emit(c);
        })
    };

    let set_current_page = {
        let scroll_to = scroll_to.clone();
        Callback::from(move |curr: usize| {
            scroll_to.emit(curr);
        })
    };

    let go_to_page = {
        let viewer = viewer.clone();
        Callback::from(move |val: usize| {
            if let Some(ref data) = &*viewer {
                data.set_current_page_number(val);
            };
        })
    };

    let toggle_thumb_sidebar = {
        let show_thumb_sidebar = show_thumb_sidebar.clone();
        Callback::from(move |e| show_thumb_sidebar.set(!(*show_thumb_sidebar)) )
    };

    let show_thumb_class = if !(*show_thumbnail_sidebar) {
        "full"
    } else {
        ""
    };


    html! {
        <div class={css!(r#"
            position: absolute;
            width: 100%;
            height: 100%;
            overflow: auto;
            "#)
        }>
            if *show_progress_bar {
                <ProgressBar progress={ *progress } />
            }
            <div class={css!(r#"
                width: 100%;
                padding-top: 50px;
                background-color: #F1F3F7;
                font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            "#)}
            >
                if *show_search_bar {
                    <SearchBar set_find_controller={set_find_controller.clone()} hide_search_bar={
                        Callback::from(move |e| show_search_bar.set(false))
                    } />
                }
                <ThumbBar
                    pdf={(*doc).clone()}
                    current_page={&(*current_page)}
                    set_current_page={ scroll_to.clone() }
                    show_thumb_sidebar={ &(*show_thumb_sidebar) }
                />
                <div class={classes!("pdfViewer", show_thumb_class)}>

                    <Toolbox
                        pdf={(*doc).clone()}
                        current_page={ *current_page }
                        set_current_page={ scroll_to.clone() }
                        { go_to_page }
                        { toggle_thumb_sidebar }
                        on_zoom_in= { 
                            let scale = scale.clone();
                            Callback::from(move |e| scale.set(*scale + ZOOM_STEP) )
                        }
                        on_zoom_out={ 
                            let scale = scale.clone();
                            Callback::from(move |e| scale.set(ZOOM_STEP - *scale) )
                        }
                        { toggle_search_bar }
                        { on_change_page }
                    />

                    <div id="pdf-pages">
                        if !url.is_empty() {
                            <PdfPages
                                url= {url.clone()}
                                scale= { &*scale }
                                { set_pdf }
                                { set_viewer }
                                {set_find_controller}
                                set_current_page={ scroll_to }
                                { set_progress }
                            />
                        }
                    </div>
                </div>
            </div>
        </div>
    }
}
