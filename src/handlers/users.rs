use crate::models::{Exercise, User};
use askama::Template;
use axum::{
    Form, Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue},
    response::{Html, IntoResponse},
    routing::{get, post},
};
use serde::Deserialize;
use sqlx::SqlitePool;
use tower_sessions::Session;

#[derive(Debug, Deserialize)]
pub struct CreateUserForm {
    pub name: String,
}

#[derive(Template)]
#[template(path = "users/list.html")]
pub struct UserListTemplate {
    pub users: Vec<User>,
    pub current_user: Option<User>,
    pub is_dashboard: bool,
}

#[derive(Template)]
#[template(path = "users/user_list_partial.html")]
pub struct UserListPartialTemplate {
    pub users: Vec<User>,
}

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub user: User,
    pub current_user: Option<User>,
    pub exercises: Vec<Exercise>,
    pub is_dashboard: bool,
}

async fn get_current_user(session: &Session, database_pool: &SqlitePool) -> Option<User> {
    if let Ok(Some(user_id)) = session.get::<String>("current_user_id").await {
        sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", user_id)
            .fetch_optional(database_pool)
            .await
            .ok()
            .flatten()
    } else {
        None
    }
}

pub async fn list_users(
    State(database_pool): State<SqlitePool>,
    session: Session,
) -> impl IntoResponse {
    let users = sqlx::query_as!(User, "select * from users;")
        .fetch_all(&database_pool)
        .await
        .unwrap_or(Vec::new());

    let current_user = get_current_user(&session, &database_pool).await;

    let template = UserListTemplate {
        users,
        current_user,
        is_dashboard: false,
    };
    Html(template.render().unwrap())
}

pub async fn create_user(
    State(database_pool): State<SqlitePool>,
    Form(form_data): Form<CreateUserForm>,
) -> impl IntoResponse {
    let new_user = User::new(form_data.name);
    sqlx::query_as!(
        User,
        "INSERT INTO users (id, name, created_at) VALUES (?, ?, ?)",
        new_user.id,
        new_user.name,
        new_user.created_at
    )
    .execute(&database_pool)
    .await
    .expect("error creating user");

    let users = sqlx::query_as!(User, "SELECT * from users")
        .fetch_all(&database_pool)
        .await
        .expect("error fetching user list");
    let template = UserListPartialTemplate {
        users: users.clone(),
    };
    Html(template.render().unwrap())
}

pub async fn select_user(
    Path(user_id): Path<String>,
    State(database_pool): State<SqlitePool>,
    session: Session,
) -> impl IntoResponse {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", user_id)
        .fetch_optional(&database_pool)
        .await
        .expect("error selecting user");

    match user {
        Some(user) => {
            session
                .insert("current_user_id", &user.id)
                .await
                .expect("Failed to insert user into session.");
            println!("user selected: {}", user.name);

            let mut headers = HeaderMap::new();
            headers.insert("HX-Redirect", HeaderValue::from_static("/dashboard"));

            (
                headers,
                Html(format!(
                    r#"<div class="bg-green-500 text-white px-4 py-2 rounded-md shadow-md">
                    ✅ {} ausgewählt! Weiterleitung...
                </div>"#,
                    user.name
                )),
            )
        }
        None => {
            let headers = HeaderMap::new();
            (
                headers,
                Html(
                    r#"<div class="bg-red-500 text-white px-4 py-2 rounded-md shadow-md">
                    ❌ User nicht gefunden!
                </div>"#
                        .to_string(),
                ),
            )
        }
    }
}

pub async fn dashboard(
    State(database_pool): State<SqlitePool>,
    session: Session,
) -> impl IntoResponse {
    match session.get::<String>("current_user_id").await {
        Ok(Some(user_id)) => {
            let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", user_id)
                .fetch_optional(&database_pool)
                .await
                .expect("Error fetching current user.");

            match user {
                Some(user) => {
                    let exercises =
                        sqlx::query_as!(Exercise, "SELECT * FROM exercises ORDER BY name LIMIT 3")
                            .fetch_all(&database_pool)
                            .await
                            .unwrap_or(Vec::new());

                    let template = DashboardTemplate {
                        user: user.clone(),
                        current_user: Some(user),
                        exercises,
                        is_dashboard: true,
                    };
                    Html(template.render().unwrap())
                }
                None => Html(
                    r#"<p>Session ungültig. <a href="/users">Bitte neu anmelden</a></p>"#
                        .to_string(),
                ),
            }
        }
        _ => Html(
            r#"<p>Du bist nicht angemeldet. <a href="/users">Jetzt anmelden</a></p>"#.to_string(),
        ),
    }
}

pub async fn logout(session: Session) -> impl IntoResponse {
    session.flush().await.expect("Failed to flush session");
    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", HeaderValue::from_static("/users"));
    (
        headers,
        Html(r#"<meta http-equiv="refresh" content="0; url=/users">"#.to_string()),
    )
}

pub fn router() -> Router<SqlitePool> {
    Router::new()
        .route("/users", get(list_users))
        .route("/users", post(create_user))
        .route("/users/{id}/select", post(select_user))
        .route("/dashboard", get(dashboard))
        .route("/logout", post(logout))
}
