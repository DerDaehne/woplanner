mod handlers;
mod models;

use axum::{Router, response::Json, routing::get};
use handlers::users::{UserStore, router as users_router};
use serde_json::{Value, json};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tower_http::services::ServeDir;

async fn hello() -> &'static str {
    "Hello, Bodybuilder!"
}

async fn health_check() -> Json<Value> {
    Json(json!({"status": "ok", "service": "woplanner"}))
}

#[tokio::main]
async fn main() {
    // initialize main user store
    let user_store: UserStore = Arc::new(Mutex::new(Vec::new()));

    let app = Router::new()
        .route("/", get(hello))
        .route("/health", get(health_check))
        .merge(users_router())
        .nest_service("/static", ServeDir::new("static"))
        .with_state(user_store);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("WOPlanner listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
