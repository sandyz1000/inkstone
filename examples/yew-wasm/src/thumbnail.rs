use std::rc::Rc;
use stylist::css;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
use yew::prelude::*;
use yew_hooks::prelude::*;

#[derive(Clone, PartialEq)]
pub struct ThumbData {
    pub thumbnail_src: String,
    pub page_num: usize,
}

async fn make_thumb(width: u32, height: u32) -> String {
    let canvas: HtmlCanvasElement;
    canvas.set_width(width);
    canvas.set_height(height);

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .into::<CanvasRenderingContext2d>()
        .unwrap();

    let render_task = page.render_to_canvas(&context);
    render_task.await.unwrap();

    canvas.to_data_url().unwrap()
}

#[derive(Properties, Clone, PartialEq)]
pub struct PdfThumbarProps {
    pub pdf: Option<PdfDocument>,
    pub current_page: usize,
    pub set_current_page: Callback<usize>,
    pub show_thumb_sidebar: bool,
}

#[function_component]
pub fn ThumbBar(
    PdfThumbarProps {
        pdf,
        current_page,
        set_current_page,
        show_thumb_sidebar,
    }: &PdfThumbarProps,
) -> Html {
    let thumbnails = use_state(|| Vec::new());
    // let show_thumbar = if !props.show_thumb_sidebar { "hide" } else { "" }

    let thumbnails = thumbnails.clone();
    use_async_with_options(
        {
            let pdf: Rc<PdfDocument> = Rc::new();
            async move {
                let num_pages = pdf.num_pages().unwrap() as usize;
                let mut pages = vec![];

                for i in 1..=num_pages {
                    pages.push(i);
                }

                let mut thumbnail_vec = vec![];

                for num in pages {
                    let page = pdf.get_page(num as u32).unwrap();
                    let thumb_url = make_thumb(&page).await;
                    thumbnail_vec.push(ThumbData {
                        thumbnail_src: thumb_url,
                        page_num: num,
                    });
                }

                thumbnails.set(thumbnail_vec);
                
                Ok::<_, ()>(())
            }
        },
        UseAsyncOptions::enable_auto(),
    );

    let class_name = css!(
        r#"
        position: fixed;
        top: 0;
        height: 100vh;
        width: 140px;
        overflow: auto;
        background-color: #F1F3F7;
        border-right: #DCE0E3 1px solid;
        padding: 22px 24px;
        transition: all 0.5s;
    
        &.hide {
            margin-left: -190px;
        }
        "#
    );

    html! {
        <div class={class_name}>
            { for thumbnails.iter().map(|thumbnail| {
                <Thumbnail
                    key={thumbnail.page_num}
                    data={thumbnail.clone()}
                    current_page={current_page}
                    set_current_page={props.set_current_page.clone()}
                />
            }) }
        </div>
    }
}

#[derive(Clone, PartialEq, Properties)]
struct ThumbnailProps {
    key: usize,
    data: ThumbData,
    set_current_page: Callback<usize>,
    current_page: usize,
}

#[function_component]
fn Thumbnail(
    ThumbnailProps {
        key,
        data,
        set_current_page,
        current_page,
    }: &ThumbnailProps,
) -> Html {
    let class_name = css!(
        r#"
        flex-direction: column;
        display: flex;
        align-items: center;
        justify-content: center;
        margin-bottom: 20px;

        img, .placeholder {
            border: solid 3px white;
            width:100%;
            cursor: pointer;        
        }
        
        .placeholder > &:hover, &.focused {
            border: solid 3px #7c86a5;
        }
        .page-number {
            margin-top: 4px;
        }
    "#
    );

    html! {
        <div class={class_name}>
            <img
                alt={data.page_num.to_string()}
                src={data.thumbnail_src.clone()}
                class={classes!(if *current_page == data.page_num { "focused" } else { "" })}
                onclick={
                    let set_current_page = set_current_page.clone();
                    let page_num = data.page_num;
                    Callback::from(move |_: MouseEvent| {
                        set_current_page.emit(page_num);
                    })
                }
            />
            <div class="page-number">{ data.page_num }</div>
        </div>
    }
}
