use axum::{
    http::{header::InvalidHeaderValue, StatusCode},
    response::{IntoResponse, Response, Json},
};
use serde_json::json;
use thiserror::Error;
use askama::Template;
use crate::templates::{ErrorPage, ErrorFragment};

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

/// Check if this is an HTMX request
pub fn is_htmx_request(headers: &axum::http::HeaderMap) -> bool {
    headers
        .get("HX-Request")
        .and_then(|v| v.to_str().ok())
        .map(|s| s == "true")
        .unwrap_or(false)
}

/// Render error to HTML using Askama templates
pub fn error_to_html(error: &AppError, headers: Option<&axum::http::HeaderMap>) -> String {
    // Determine if this is an HTMX request
    let is_htmx = headers.map(|h| is_htmx_request(h)).unwrap_or(false);
    
    match error {
        AppError::Database(_) => {
            let fragment = ErrorFragment {
                message: "Datenbankfehler. Bitte versuchen Sie es später erneut.".to_string(),
            };
            if is_htmx {
                fragment.render().unwrap_or_else(|_| "Database error.".to_string())
            } else {
                let page = ErrorPage {
                    status_code: 500,
                    message: "Internal Server Error".to_string(),
                    detail: "Database operation failed. Please try again later.".to_string(),
                    current_user: None,
                    is_dashboard: false,
                };
                page.render().unwrap_or_else(|_| "Database error.".to_string())
            }
        }
        AppError::Template(_) => {
            let fragment = ErrorFragment {
                message: "Template rendering error.".to_string(),
            };
            if is_htmx {
                fragment.render().unwrap_or_else(|_| "Template error.".to_string())
            } else {
                let page = ErrorPage {
                    status_code: 500,
                    message: "Template Error".to_string(),
                    detail: "Failed to render template.".to_string(),
                    current_user: None,
                    is_dashboard: false,
                };
                page.render().unwrap_or_else(|_| "Template error.".to_string())
            }
        }
        AppError::InvalidHeader(_) => {
            let fragment = ErrorFragment {
                message: "Invalid header value.".to_string(),
            };
            if is_htmx {
                fragment.render().unwrap_or_else(|_| "Invalid header.".to_string())
            } else {
                let page = ErrorPage {
                    status_code: 400,
                    message: "Bad Request".to_string(),
                    detail: "Invalid response header.".to_string(),
                    current_user: None,
                    is_dashboard: false,
                };
                page.render().unwrap_or_else(|_| "Invalid header.".to_string())
            }
        }
        AppError::Session(_) => {
            let fragment = ErrorFragment {
                message: "Session error.".to_string(),
            };
            if is_htmx {
                fragment.render().unwrap_or_else(|_| "Session error.".to_string())
            } else {
                let page = ErrorPage {
                    status_code: 500,
                    message: "Session Error".to_string(),
                    detail: "Session error occurred.".to_string(),
                    current_user: None,
                    is_dashboard: false,
                };
                page.render().unwrap_or_else(|_| "Session error.".to_string())
            }
        }
        AppError::Unauthorized => {
            let fragment = ErrorFragment {
                message: "Not authenticated.".to_string(),
            };
            if is_htmx {
                fragment.render().unwrap_or_else(|_| "Unauthorized.".to_string())
            } else {
                let page = ErrorPage {
                    status_code: 401,
                    message: "Unauthorized".to_string(),
                    detail: "Please log in to continue.".to_string(),
                    current_user: None,
                    is_dashboard: false,
                };
                page.render().unwrap_or_else(|_| "Unauthorized.".to_string())
            }
        }
        AppError::NotFound(msg) => {
            let fragment = ErrorFragment {
                message: format!("Not found: {}", msg),
            };
            if is_htmx {
                fragment.render().unwrap_or_else(|_| "Not found.".to_string())
            } else {
                let page = ErrorPage {
                    status_code: 404,
                    message: "Not Found".to_string(),
                    detail: msg.clone(),
                    current_user: None,
                    is_dashboard: false,
                };
                page.render().unwrap_or_else(|_| "Not found.".to_string())
            }
        }
        AppError::BadRequest(msg) => {
            let fragment = ErrorFragment {
                message: format!("Bad request: {}", msg),
            };
            if is_htmx {
                fragment.render().unwrap_or_else(|_| "Bad request.".to_string())
            } else {
                let page = ErrorPage {
                    status_code: 400,
                    message: "Bad Request".to_string(),
                    detail: msg.clone(),
                    current_user: None,
                    is_dashboard: false,
                };
                page.render().unwrap_or_else(|_| "Bad request.".to_string())
            }
        }
        AppError::Internal(msg) => {
            let fragment = ErrorFragment {
                message: "Internal error occurred.".to_string(),
            };
            if is_htmx {
                fragment.render().unwrap_or_else(|_| "Internal error.".to_string())
            } else {
                let page = ErrorPage {
                    status_code: 500,
                    message: "Internal Error".to_string(),
                    detail: msg.clone(),
                    current_user: None,
                    is_dashboard: false,
                };
                page.render().unwrap_or_else(|_| "Internal error.".to_string())
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // For now, return JSON as default - this will be updated when
        // we have proper middleware to detect request context
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




