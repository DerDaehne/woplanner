use askama::Template;

#[derive(Template)]
#[template(path = "error_page.html")]
pub struct ErrorPage {
    pub status_code: u16,
    pub message: String,
    pub detail: String,
}
