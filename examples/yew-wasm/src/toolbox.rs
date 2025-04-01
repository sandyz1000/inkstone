#![allow(unused)]

use stylist::{css, yew::styled_component};
use web_sys::{HtmlElement, HtmlInputElement};
use yew::prelude::*;
use yew_hooks::prelude::*;

use super::{error::ApiError};

#[derive(Properties, Clone, PartialEq)]
pub struct ToolboxProps {
    pub pdf: Option<PdfDocument>,
    pub current_page: usize,
    pub set_current_page: Callback<usize>,
    pub go_to_page: Callback<usize>,
    pub toggle_thumb_sidebar: Callback<MouseEvent>,
    pub on_zoom_in: Callback<MouseEvent>,
    pub on_zoom_out: Callback<MouseEvent>,
    pub toggle_search_bar: Callback<MouseEvent>,
    pub on_change_page: Callback<usize>,
}

#[styled_component]
pub fn Toolbox(
    ToolboxProps {
        pdf,
        current_page,
        set_current_page,
        go_to_page,
        toggle_thumb_sidebar,
        on_zoom_in,
        on_zoom_out,
        toggle_search_bar,
        on_change_page,
    }: &ToolboxProps,
) -> Html {
    let page_input_focus = use_state(|| false);
    let current_page_ref = use_node_ref();
    
    let toggle_focus = {
        let page_input_focus = page_input_focus.clone();
        Callback::from(move |e: FocusEvent| {
            page_input_focus.set(!(*page_input_focus));
        })
    };

    let page_up = {
        let current_page = *current_page;
        let set_current_page = set_current_page.clone();
        let go_to_page = go_to_page.clone();
        Callback::from(move |e: MouseEvent| {
            if current_page != 1 {
                let new_page = current_page - 1;
                set_current_page.emit(new_page);
                go_to_page.emit(new_page);
            }
        })
    };

    let page_down = {
        let current_page = *current_page;
        let set_current_page = set_current_page.clone();
        let go_to_page = go_to_page.clone();
        let Some(doc) = pdf.to_owned() else {
            panic!("No pdf docs found");
        };
            
        Callback::from(move |e: MouseEvent| {
                
            if current_page != doc.num_pages {
                let new_page = current_page + 1;
                set_current_page.emit(new_page);
                go_to_page.emit(new_page);
            }
        })
    };
    
    let num_pages = if let Some(doc) = pdf {
        doc.num_pages
    } else {
        0
    };

    html! {
        <>
        <div class={css!(r#"
            position: relative;
            width: 100%;
            height: 100%;
        "#)}>
        <div class={ css!(r#"
            position: absolute;
            top: calc(100vh - 120px);
            left: calc(50% - 208px);
        "#) }>
        <div class={css!(r#"
            position: fixed;
            background-color: #97a1b6;
            z-index: 9999;
            bottom: 28px;
            max-height: 48px;
            width: 416px;
            padding: 12px 20px;
            display: flex;
            align-items: center;
            justify-content: space-between;
            color: white;
            border-radius: 3px;
        "#)}>
            <Thumbnail id="thumbnail-icon" onclick={toggle_thumb_sidebar} />

            <div class={css!("display: flex; font-size: 13px;")}>
              <PageUp
                id="page-up"
                onclick={page_up}
                class_name={classes!(css!("margin-right: 20px;"), if *current_page == 1  {"disabled"} else {""} ) }
              />

              <span
                class={
                    classes!(css!(r#"
                        font-size: 13px;
                        font-weight: 500;
                        color: white;
                        overflow: visible;
                        border-bottom: white 1px solid;
                        "#), 
                    if *page_input_focus { css!("border-bottom: none;")} else {css!("")} )
                }
                onfocus={
                    let page_input_focus = page_input_focus.clone();
                    Callback::from(move |e: FocusEvent| page_input_focus.set(true))
                }
                onclick={
                    let page_input_focus = page_input_focus.clone();
                    Callback::from(move |e: MouseEvent| page_input_focus.set(true))
                }
              >
                <input
                    class={ css!(r#"
                        font-size: 13px;
                        text-align: center;
                        width: 20px;
                        height: 15px;
                        font-weight: 500;
                        color: white;
                        border: none;
                        outline: none;
                        background-color: #97a1b6;

                        &:focus {
                        border:1px solid #D0DAE3;
                        border-radius: 2px;
                        height: 20px;
                        }
                    "#) }
                    ref={current_page_ref}
                    type="number"
                    value={(*current_page).to_string()}
                    min={1} max={num_pages.to_string()}
                    onchange={
                        let on_change_page = on_change_page.clone();
                        Callback::from(move |e| on_change_page.emit(1) )
                    }
                    onblur={
                        Callback::from(move |e: FocusEvent| page_input_focus.set(false))
                    }
                />

                <span> {num_pages} </span>
              </span>

              <PageDown
                id="page-down"
                onclick={page_down}
                class_name={classes!(css!("margin-right: 20px;"), if *current_page == num_pages {"disabled"} else {""} )}
              />
            </div>

            <ZoomIn id="zoom-in" onclick={on_zoom_in} />
            <ZoomOut id="zoom-out" onclick={on_zoom_out} />
            <SearchIcon id="search-icon" onclick={toggle_search_bar} />
          </div>
        </div>
      </div>
      </>
    }
}

#[derive(Properties, PartialEq)]
struct SearchIconProps {
    id: String,
    onclick: Callback<MouseEvent>,
}

#[function_component]
fn SearchIcon(SearchIconProps { id, onclick }: &SearchIconProps) -> Html {
    html! {
        <svg width={24} height={24} fill="none" view-box="0 0 24 24" {onclick} id={id.clone()} >
            <path
            fill="#fff"
            fill-rule="evenodd"
            d="M10 18a8 8 0 116.32-3.094l5.387 5.387-1.414 1.414-5.387-5.387A7.969 7.969 0 0110 18zm6-8a6 6 0 11-12 0 6 6 0 0112 0z"
            clip-rule="evenodd"
            />
        </svg>
    }
}

#[derive(Properties, PartialEq)]
struct ZoomProps {
    id: String,
    onclick: Callback<MouseEvent>,
}

#[function_component]
fn ZoomOut(ZoomProps { id, onclick }: &ZoomProps) -> Html {
    html! {
        <svg width={24} height={24} fill="none" view-box="0 0 24 24" {onclick} id={id.clone()} >
            <path
            fill="#fff"
            fill-rule="evenodd"
            d="M2 10a8 8 0 0012.906 6.32l5.387 5.387 1.414-1.414-5.387-5.387A8 8 0 102 10zm8 6a6 6 0 100-12 6 6 0 000 12zM6 9v2h8V9H6z"
            clip-rule="evenodd"
            />
        </svg>
    }
}

#[function_component]
fn ZoomIn(ZoomProps { id, onclick }: &ZoomProps) -> Html {
    html! {
        <svg width={24} height={24} fill="none" viewBox="0 0 24 24" {onclick} id={id.clone()} >
            <path
            fill="#fff"
            fill-rule="evenodd"
            d="M2 10a8 8 0 0012.906 6.32l5.387 5.387 1.414-1.414-5.387-5.387A8 8 0 102 10zm8 6a6 6 0 100-12 6 6 0 000 12zM9 6v3H6v2h3v3h2v-3h3V9h-3V6H9z"
            clip-rule="evenodd"
            />
        </svg>
    }
}

#[derive(Properties, PartialEq)]
struct PageNavProps {
    id: String,
    onclick: Callback<MouseEvent>,
    class_name: Classes,
}

#[function_component]
fn PageUp(
    PageNavProps {
        id,
        onclick,
        class_name,
    }: &PageNavProps,
) -> Html {
    html! {
        <svg
            width={24}
            height={24}
            fill="none"
            view-box="0 0 24 24"
            {onclick}
            class={class_name.clone()}
            id={id.clone()}
        >
            <path
            fill="#fff"
            fill-rule="evenodd"
            d="M12 9.414l-7.293 7.293-1.414-1.414L12 6.586l8.707 8.707-1.414 1.414L12 9.414z"
            clip-rule="evenodd"
            />
        </svg>
    }
}

#[function_component]
fn PageDown(
    PageNavProps {
        id,
        onclick,
        class_name,
    }: &PageNavProps,
) -> Html {
    html! {
        <svg width={24} height={24} fill="none" view-box="0 0 24 24"
            {onclick}
            class={class_name.clone()}
            id={id.clone()}
        >
            <path
            fill="#fff"
            fill-rule="evenodd"
            d="M19.293 7.293l1.414 1.414L12 17.414 3.293 8.707l1.414-1.414L12 14.586l7.293-7.293z"
            clip-rule="evenodd"
            />
        </svg>
    }
}


#[derive(Properties, PartialEq)]
struct ThumbnailProps {
    id: String,
    onclick: Callback<MouseEvent>
}

#[function_component]
fn Thumbnail(ThumbnailProps{ id, onclick }: &ThumbnailProps) -> Html {
    html! {
        <svg width={24} height={24} fill="none" viewBox="0 0 24 24" id={ id.clone() } {onclick}>
            <path
                fill="#fff"
                fillRule="evenodd"
                d="M19 5h-8v15h8V5zM4 5h5v15H4V5zm0-2a2 2 0 00-2 2v15a2 2 0 002 2h15a2 2 0 002-2V5a2 2 0 00-2-2H4zm2 3a1 1 0 00-1 1v1a1 1 0 001 1h1a1 1 0 001-1V7a1 1 0 00-1-1H6zm0 5a1 1 0 00-1 1v1a1 1 0 001 1h1a1 1 0 001-1v-1a1 1 0 00-1-1H6zm-1 6a1 1 0 011-1h1a1 1 0 011 1v1a1 1 0 01-1 1H6a1 1 0 01-1-1v-1z"
                clipRule="evenodd"
            />
        </svg>
    }
}
