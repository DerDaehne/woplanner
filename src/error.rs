use axum::{
    http::{header::InvalidHeaderValue, HeaderValue, StatusCode},
    response::{IntoResponse, Response, Html, Json},
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Template render error: {0}")]
    Template(#[from] askama::Error),

    #[error("Invalid header value: {0}")]
    InvalidHeader(#[from] InvalidHeaderValue),

    #[error("Session error: {0}")]
    Session(#[from] tower_sessions::session::Error),

    #[error("Not authenticated")]
    Unauthorized,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Database(e) => {
                tracing::error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                    "error": "Database operation failed"
                }))).into_response()
            }
            AppError::Template(e) => {
                tracing::error!("Template render error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                    "error": "Failed to render template"
                }))).into_response()
            }
            AppError::InvalidHeader(e) => {
                tracing::warn!("Invalid header value: {}", e);
                (StatusCode::BAD_REQUEST, Json(json!({
                    "error": "Invalid response header"
                }))).into_response()
            }
            AppError::Session(e) => {
                tracing::error!("Session error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                    "error": "Session error"
                }))).into_response()
            }
            AppError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, Json(json!({
                    "error": "Not authenticated"
                }))).into_response()
            }
            AppError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, Json(json!({
                    "error": msg
                }))).into_response()
            }
            AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, Json(json!({
                    "error": msg
                }))).into_response()
            }
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                    "error": "An internal error occurred"
                }))).into_response()
            }
        }
    }
}

// HTMX-specific error response
impl AppError {
    pub fn to_htmx_redirect(&self, url: &str) -> Response {
        let mut headers = axum::http::HeaderMap::new();
        headers.insert(
            "HX-Redirect",
            HeaderValue::from_str(url).unwrap(),
        );
        (StatusCode::SEE_OTHER, headers, Html(self.to_string())).into_response()
    }
}

// Convert AppError to a plain text/html response for non-JSON endpoints
pub fn error_to_html(error: &AppError) -> String {
    match error {
        AppError::Database(_) => "Database error. Please try again later.".to_string(),
        AppError::Template(_) => "Template rendering error.".to_string(),
        AppError::InvalidHeader(_) => "Invalid header value.".to_string(),
        AppError::Session(_) => "Session error.".to_string(),
        AppError::Unauthorized => "Not authenticated.".to_string(),
        AppError::NotFound(msg) => format!("Not found: {}", msg),
        AppError::BadRequest(msg) => format!("Bad request: {}", msg),
        AppError::Internal(_) => "An internal error occurred.".to_string(),
    }
}
