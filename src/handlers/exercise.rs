use crate::models::{Exercise, User};
use askama::Template;
use axum::{
    Form, Router,
    extract::State,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use serde::Deserialize;
use sqlx::SqlitePool;
use tower_sessions::Session;

#[derive(Debug, Deserialize)]
pub struct CreateExerciseForm {
    pub name: String,
    pub instructions: String,
    pub video_url: Option<String>,
}

#[derive(Template)]
#[template(path = "exercises/list.html")]
pub struct ExerciseListTemplate {
    pub exercises: Vec<Exercise>,
    pub current_user: Option<User>,
    pub is_dashboard: bool,
}

#[derive(Template)]
#[template(path = "exercises/exercise_list_partial.html")]
pub struct ExerciseListPartialTemplate {
    pub exercises: Vec<Exercise>,
}

async fn get_current_user(session: &Session, pool: &SqlitePool) -> Option<User> {
    if let Ok(Some(user_id)) = session.get::<String>("current_user_id").await {
        sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", user_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
    } else {
        None
    }
}

pub async fn list_exercises(
    State(database_pool): State<SqlitePool>,
    session: Session,
) -> impl IntoResponse {
    let exercises = sqlx::query_as!(Exercise, "SELECT id, name, instructions, video_url, created_at FROM exercises ORDER BY name")
        .fetch_all(&database_pool)
        .await
        .unwrap_or(Vec::new());

    let current_user = get_current_user(&session, &database_pool).await;

    let template = ExerciseListTemplate {
        exercises,
        current_user,
        is_dashboard: false,
    };

    Html(template.render().unwrap())
}

pub async fn create_exercise(
    State(database_pool): State<SqlitePool>,
    Form(form_data): Form<CreateExerciseForm>,
) -> impl IntoResponse {
    // Normalize empty string to None for video_url
    let video_url = form_data.video_url.filter(|url| !url.trim().is_empty());
    let new_exercise = Exercise::new(form_data.name, form_data.instructions, video_url);

    sqlx::query!(
        "INSERT INTO exercises (id, name, instructions, video_url, created_at) VALUES (?, ?, ?, ?, ?)",
        new_exercise.id,
        new_exercise.name,
        new_exercise.instructions,
        new_exercise.video_url,
        new_exercise.created_at
    )
    .execute(&database_pool)
    .await
    .expect("error creating new exercise");

    let exercises = sqlx::query_as!(Exercise, "SELECT id, name, instructions, video_url, created_at FROM exercises ORDER BY name")
        .fetch_all(&database_pool)
        .await
        .expect("error fetching exercise list");

    let template = ExerciseListPartialTemplate { exercises };
    Html(template.render().unwrap())
}

pub fn router() -> Router<SqlitePool> {
    Router::new()
        .route("/exercises", get(list_exercises))
        .route("/exercises", post(create_exercise))
}
