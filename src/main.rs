mod database;
mod handlers;
mod models;

use axum::{
    Router,
    response::{Html, Json},
    routing::get,
};
use handlers::exercise::router as exercise_router;
use handlers::users::router as users_router;
use serde_json::{Value, json};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tower_sessions::Session;
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store_chrono::SqliteStore;

async fn root(session: Session) -> Html<String> {
    match session.get::<String>("current_user_id").await {
        Ok(Some(_)) => {
            Html(r#"<meta http-equiv="refresh" content="0; url=/dashboard">"#.to_string())
        }
        _ => Html(r#"<meta http-equiv="refresh" content="0; url=/users">"#.to_string()),
    }
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

    let session_store = SqliteStore::new(database_pool.clone());
    session_store
        .migrate()
        .await
        .expect("Failed to migrate sessions");

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::hours(24)));

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .merge(users_router())
        .merge(exercise_router())
        .nest_service("/static", ServeDir::new("static"))
        .layer(session_layer)
        .with_state(database_pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("WOPlanner listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
