#![allow(unused)]

use serde::{Deserialize, Serialize};
use gloo_utils::format::json::JsValueSerdeExt;
use serde_json::json;
use std::time::Duration;
use stylist::{css, yew::styled_component};
use wasm_bindgen::{JsValue, JsCast};
use web_sys::{HtmlElement, HtmlInputElement};
use yew::prelude::*;
use crate::types::PDFFindController;


#[derive(Properties, PartialEq)]
pub struct SearchBarProps {
    pub set_find_controller: Callback<PDFFindController>,
    pub hide_search_bar: Callback<bool>,
}

#[styled_component]
fn SearchInfo() -> Html {
    let styles = stylist::css!(
        r#"
        font-size: 0.813rem;
        display: flex;
        align-items: center;

        .search-status {
            display: flex;
            align-items: center;
            color: rgba(151, 161, 182, 0.5);
        }

        cursor: pointer;
        margin-left: 4px;

        &.disabled {
            cursor: default;
        }
    "#
    );

    html! {
        <div class={styles}>
        </div>
    }
}

#[derive(Debug)]
struct State {
    search_term: String,
    current_match_index: u32,
    match_count: u32,
    search_completed: bool,
}

#[derive(Debug, Serialize)]
pub struct FindParams {
    case_sensitive: bool,
    find_previous: Option<bool>,
    highlight_all: bool,
    phrase_search: bool,
    query: String,
}

#[derive(Serialize, Deserialize)]
struct PDFControllerArgs {
    // event: ControllerEvent
}

#[function_component]
pub fn SearchBar(props: &SearchBarProps) -> Html {
    let data = PDFControllerArgs {};

    let search_input_ref = use_node_ref();
    let controller_event: ControllerEvent;
    let params = JsValue::from_serde(&data).unwrap();
    let find_controller = PDFFindController::new(&params);
    let state: UseStateHandle<Option<State>> = use_state(|| None::<State>);

    let search_input = use_node_ref();

    let on_search = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            controller_event.dispatch("find".to_string(), todo!());
        })
    };

    // TODO: Serialize and cast to JsValue object
    // &FindParams {
    //     case_sensitive: false,
    //     find_previous: Some(find_previous),
    //     highlight_all: true,
    //     phrase_search: true,
    //     query: self.search_term.clone(),
    // },
    let on_find_again = {
        let search_term = state.clone();
        Callback::from(move |e: InputEvent| {
            controller_event.dispatch("findagain".to_string(), todo!());
        })
    };

    let on_search_term = {
        let state = state.clone();
        Callback::from(move |e: Event| {
            let term = "".to_string();
            if let Some(mut state) = &*state {
                state.search_term = term.clone();
                state.search_completed = false;
            }
            controller_event.dispatch("findagain".to_string(), todo!())
        })
    };

    let on_search_next = { Callback::from(move |e: KeyboardEvent| {}) };

    let on_search_complete = {
        let state = state.clone();
        let find_controller = find_controller.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(mut data) = &*state {
                // data.match_count = find_controller.match_count();
                // data.current_match_index = find_controller.current_match_index();
                data.search_completed = true;
            }
        })
    };

    let previous_match = {
        Callback::from(move |e: MouseEvent| {

        })
    };

    let next_match = {
        Callback::from(move |e: MouseEvent| {

        })
    };

    let on_exit_search = Callback::from({
        let state = state.clone();
        move |e: MouseEvent| {
            controller_event.dispatch("find".to_string(), todo!());
            if let Some(mut data) = &*state {
                data.search_completed = false;
                data.match_count = 0;
                props.hide_search_bar.emit(true);
            }
        }
    });
    let Some(data) = &*state else {
        panic!("Error!!!");
    };

    html! {
        <div id="pdfSearchbar" class={css!(r#"
            position: fixed;
            display: flex;
            align-items: center;
            justify-content: space-between;
            top: 20px;
            left: 200px;
            width: 300px;
            max-height: 36px;
            border-radius: 3px;
            background-color: #313b51;
            z-index: 9999;
            padding: 10px;
        "#)}>
            <SearchBarInput
                search_input={search_input_ref}
                placeholder="Search in document"
                on_search_term={on_search_term}
                auto_focus=true
                on_key_down={on_search_next}
            />
            <div class={css!("font-size: 0.813rem; display: flex; align-items: center;")}>

                if !data.search_term.is_empty() {
                    <>
                    <span class="search-status">

                        if data.search_completed {
                            { format!("{}/{}", data.current_match_index, data.match_count) }
                        }

                        if data.match_count > 0 {
                            <>
                                <PrevIcon onclick={previous_match} disabled={ true } />
                                <NextIcon onclick={next_match} disabled={ true } />
                            </>
                        }
                    </span>
                    </>
                }

                <CloseIcon id="close-icon" class="search-bar-ico" onclick={on_exit_search} />
            </div>
        </div>
    }
}

#[derive(Debug, Properties, PartialEq)]
struct SearchBarInputProps {
    search_input: NodeRef,
    placeholder: String,
    auto_focus: bool,
    on_search_term: Callback<Event>,
    on_key_down: Callback<KeyboardEvent>,
}

#[function_component]
fn SearchBarInput(
    SearchBarInputProps {
        search_input,
        placeholder,
        auto_focus,
        on_search_term,
        on_key_down,
    }: &SearchBarInputProps,
) -> Html {
    let class_name = css!(
        r#"    
        font-size: 0.813rem;
        background-color: #313b51;
        border: none;
        color: white;

        &::placeholder {
            color: rgba(151, 161, 182, 0.5);
        }

        &:focus {
            outline: none !important;
            border: none;
        }
    "#
    );

    html! {
        <input
          ref={search_input}
          class={ class_name }
          auto_focus={auto_focus.to_string()}
          placeholder={placeholder.clone()}
          onchange={ on_search_term.clone() }
          onkeydown={ on_key_down }
        />
    }
}

#[derive(Debug, Properties, PartialEq)]
struct IconProps {
    disabled: bool,
    onclick: Callback<MouseEvent>,
}

#[derive(Debug, Properties, PartialEq)]
struct CloseIconProp {
    id: String,
    class: String,
    onclick: Callback<MouseEvent>,
}

#[function_component]
fn NextIcon(IconProps { disabled, onclick }: &IconProps) -> Html {
    let class_name = classes!(
        css!(r#" cursor: pointer; margin-left: 4px;"#),
        if *disabled {
            css!(r#"cursor: default; path { fill-opacity: 0.29; }"#)
        } else {
            css!("")
        }
    );
    html! {
        <>
        <svg width={18} height={18} fill="none" view-box="0 0 18 18" class={class_name} onclick={ onclick }>
            <path
            fill="#E3E8EF"
            fill-rule="evenodd"
            d="M10.94 9L5.47 3.53l1.06-1.06L13.06 9l-6.53 6.53-1.06-1.06L10.94 9z"
            clip-rule="evenodd"
            />
        </svg>      
        </>
    }
}

#[styled_component]
fn PrevIcon(IconProps { disabled, onclick }: &IconProps) -> Html {
    let class_name = classes!(
        css!(r#" cursor: pointer; margin-left: 4px;"#),
        if *disabled {
            css!(r#"cursor: default; path { fill-opacity: 0.29; }"#)
        } else {
            css!("")
        }
    );
    html! {
        <>
        <svg width={18} height={18} fill="none" view-box="0 0 18 18" class={class_name} onclick={ onclick }>
            <path
            fill="#E3E8EF"
            fill-rule="evenodd"
            d="M10.94 9L5.47 3.53l1.06-1.06L13.06 9l-6.53 6.53-1.06-1.06L10.94 9z"
            clip-rule="evenodd"
            />
        </svg>      
        </>
    }
}

#[styled_component]
fn CloseIcon(props: &CloseIconProp) -> Html {
    let id = props.id.clone();
    let icon_class = props.class.clone();
    let on_click = props.onclick.clone();

    html! {
        <div id={id} class={icon_class} onclick={ on_click }>

        </div>
    }
}
