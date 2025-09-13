use crate::models::User;
use axum::{
    Form, Router,
    extract::State,
    response::Html,
    routing::{get, post},
};
use serde::Deserialize;
use std::sync::{Arc, Mutex};

pub type UserStore = Arc<Mutex<Vec<User>>>;

#[derive(Debug, Deserialize)]
pub struct CreateUserForm {
    pub name: String,
}

pub async fn list_users(State(store): State<UserStore>) -> Html<String> {
    let users = store.lock().unwrap();

    let mut html = String::from(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
          <title>WOPlanner - User</title>
          <meta charset="UTF-8">
        </head>
        <body>
            <h1> Wer trainiert heute? 💪</h1>
            <div class="user-list">
        "#,
    );

    if users.is_empty() {
        html.push_str("<p>Noch kein User vorhanden.</p>");
    } else {
        html.push_str("<ul>");
        for user in users.iter() {
            html.push_str(&format!(
                r#"<li>
                <strong>{}</strong>
                <small>(ID: {})</small>
                <button onclick="selectUser('{}')">Auswählen</button>
               </li>"#,
                user.name, user.id, user.id
            ));
        }
        html.push_str("</ul>");
    }

    html.push_str(
        r#"
        </div>
        <hr>
        <h2>Neuen User hinzufuegen</h2>
        <form method="POST" action="/users">
            <input type="text" name="name" placeholder="Name eingeben" required>
            <button type="submit">User hinzufuegen</button>
        </form>
        <script>
            function selectUser(userId) {
                alert('User ' + userId + ' ausgewaehlt! (TODO: session setzen)');
            }
        </script>
        </body>
        </html>
        "#,
    );

    Html(html)
}

pub async fn create_user(
    State(store): State<UserStore>,
    Form(form_data): Form<CreateUserForm>,
) -> Html<String> {
    let new_user = User::new(form_data.name);

    {
        let mut users = store.lock().unwrap();
        users.push(new_user);
        println!("user addes successfully. total: {}", users.len());
    }

    Html(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta http-equiv="refresh" content="0; url=/users">
            <title>Redirecting...</title>
        </head>
        <body>
            <p>User wurde hinzugefuegt! <a href="/users">Zurueck zur Liste</a></p>
        </body>
        </html>
        "#
        .to_string(),
    )
}

pub fn router() -> Router<UserStore> {
    Router::new()
        .route("/users", get(list_users))
        .route("/users", post(create_user))
}
