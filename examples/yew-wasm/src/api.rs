use url::Url;
use web_sys::js_sys::Error;



#[derive(Debug, Clone)]
pub struct PdfController;

fn stream(url: Url) {
    todo!()
}

pub async fn find(
    case_sensitive: bool,
    find_previous: bool,
    highlight_all: bool,
    phrase_search: bool,
    query: String,
) -> Result<(), Error> {
    todo!()
}

pub async fn findagain(
    case_sensitive: bool,
    find_previous: bool,
    highlight_all: bool,
    phrase_search: bool,
    query: String,
) -> Result<(), Error> {
    todo!()
}

pub async fn on_toggle_thumbnail(show_sidebar: bool) -> Result<(), Error> {
    todo!()
}

async fn on_progress() -> Result<(), Error> {
    todo!()
}

async fn on_zoom_in() -> Result<(), Error> {
    todo!()
}

async fn on_zoom_out() -> Result<(), Error> {
    todo!()
}
