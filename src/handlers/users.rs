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
) -> Html<String> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", user_id)
        .fetch_optional(&database_pool)
        .await
        .expect("error selecting user");

    println!("{}", user_id);
    println!("{:?}", user);

    match user {
        Some(user) => {
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

pub fn router() -> Router<SqlitePool> {
    Router::new()
        .route("/users", get(list_users))
        .route("/users", post(create_user))
        .route("/users/{id}/select", post(select_user))
}
