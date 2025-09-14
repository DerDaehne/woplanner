use crate::models::User;
use askama::Template;
use axum::{
    Form, Router,
    extract::{Path, State},
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
}

#[derive(Template)]
#[template(path = "users/user_list_partial.html")]
pub struct UserListPartialTemplate {
    pub users: Vec<User>,
}

pub async fn list_users(State(database_pool): State<SqlitePool>) -> impl IntoResponse {
    let users = sqlx::query_as!(User, "select * from users;")
        .fetch_all(&database_pool)
        .await
        .unwrap_or(Vec::new());
    let template = UserListTemplate { users };
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
) -> Html<String> {
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
            Html(format!(
                r#"<div class="bg-green-500 text-white px-4 py-2 rounded-md shadow-md">
                    ✅ {} ausgewählt!
                </div>"#,
                user.name
            ))
        }
        None => Html(
            r#"<div class="bg-red-500 text-white px-4 py-2 rounded-md shadow-md">
                ❌ User nicht gefunden!
            </div>"#
                .to_string(),
        ),
    }
}

pub async fn dashboard(State(database_pool): State<SqlitePool>, session: Session) -> Html<String> {
    match session.get::<String>("current_user_id").await {
        Ok(Some(user_id)) => {
            let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", user_id)
                .fetch_optional(&database_pool)
                .await
                .expect("Error fetching current user.");

            match user {
                Some(user) => Html(format!(
                    r#"<h1>Willkommen zurück, {}! 💪</h1>
                    <p>Du bist eingeloggt und bereit zum Training!</p>
                    <a href="/users" class="bg-blue-500 text-white px-4 py-2 rounded">User wechseln</a>"#,
                    user.name
                )),
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

pub fn router() -> Router<SqlitePool> {
    Router::new()
        .route("/users", get(list_users))
        .route("/users", post(create_user))
        .route("/users/{id}/select", post(select_user))
        .route("/dashboard", get(dashboard))
}
