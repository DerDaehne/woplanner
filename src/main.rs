mod database;
mod handlers;
mod models;

use axum::{Router, response::Json, routing::get};
use handlers::users::router as users_router;
use serde_json::{Value, json};
use std::net::SocketAddr;
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
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./.sqlite".to_string());
    let database_pool = database::create_database_pool(&database_url)
        .await
        .expect("error: can't connect to database!");

    let app = Router::new()
        .route("/", get(hello))
        .route("/health", get(health_check))
        .merge(users_router())
        .nest_service("/static", ServeDir::new("static"))
        .with_state(database_pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("WOPlanner listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
