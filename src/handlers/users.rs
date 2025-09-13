use crate::models::User;
use askama::Template;
use axum::{
    Form, Router,
    extract::{Path, State},
    response::{Html, IntoResponse},
    routing::{get, post},
};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub type UserStore = Arc<Mutex<Vec<User>>>;

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

pub async fn list_users(State(store): State<UserStore>) -> impl IntoResponse {
    let users = store.lock().unwrap();
    let template = UserListTemplate {
        users: users.clone(),
    };
    Html(template.render().unwrap())
}

pub async fn create_user(
    State(store): State<UserStore>,
    Form(form_data): Form<CreateUserForm>,
) -> impl IntoResponse {
    let new_user = User::new(form_data.name);

    {
        let mut users = store.lock().unwrap();
        users.push(new_user);
        println!("user addes successfully. total: {}", users.len());
    }

    let users = store.lock().unwrap();
    let template = UserListPartialTemplate {
        users: users.clone(),
    };
    Html(template.render().unwrap())
}

pub async fn select_user(
    Path(user_id): Path<Uuid>,
    State(store): State<UserStore>,
) -> Html<String> {
    let users = store.lock().unwrap();
    let user = users.iter().find(|u| u.id == user_id);

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

pub fn router() -> Router<UserStore> {
    Router::new()
        .route("/users", get(list_users))
        .route("/users", post(create_user))
        .route("/users/{id}/select", post(select_user))
}
