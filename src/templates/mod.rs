use askama::Template;
use crate::models::User;

#[derive(Template)]
#[template(path = "error_page.html")]
pub struct ErrorPage {
    pub status_code: u16,
    pub message: String,
    pub detail: String,
    pub current_user: Option<User>,
    pub is_dashboard: bool,
}

#[derive(Template)]
#[template(path = "error_fragment.html")]
pub struct ErrorFragment {
    pub message: String,
}
