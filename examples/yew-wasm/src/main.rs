mod api;
mod error;
mod pages;
mod progressbar;
mod searchbar;
mod thumbnail;
mod toolbox;
mod view;
mod context;
mod types;

use yew::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        "Render pdf with yew!!"
    }
}


fn main() {
    yew::Renderer::<App>::new().render();
}
