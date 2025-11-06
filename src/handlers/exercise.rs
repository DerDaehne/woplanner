use crate::models::{Exercise, User};
use askama::Template;
use axum::{
    Form, Router,
    extract::{Path, State},
    response::{Html, IntoResponse},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize)]
pub struct ProgressionDataPoint {
    pub date: String,
    pub weight: Option<f32>,
    pub reps: i32,
    pub volume: Option<f32>,
    pub set_number: i32,
}

#[derive(Template)]
#[template(path = "exercises/progression.html")]
pub struct ExerciseProgressionTemplate {
    pub exercise: Exercise,
    pub progression_data: Vec<ProgressionDataPoint>,
    pub progression_data_json: String, // JSON string for JavaScript
    pub current_user: Option<User>,
    pub is_dashboard: bool,
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

pub async fn show_exercise_progression(
    Path(exercise_id): Path<String>,
    State(database_pool): State<SqlitePool>,
    session: Session,
) -> impl IntoResponse {
    let current_user = get_current_user(&session, &database_pool).await;

    // Get the exercise
    let exercise = match sqlx::query_as!(
        Exercise,
        "SELECT id, name, instructions, video_url, created_at FROM exercises WHERE id = ?",
        exercise_id
    )
    .fetch_optional(&database_pool)
    .await
    {
        Ok(Some(ex)) => ex,
        _ => return Html("Exercise not found".to_string()).into_response(),
    };

    // Get progression data for this exercise from completed sets
    // We'll get the last 50 sets to show recent progression
    let user_id = match &current_user {
        Some(user) => &user.id,
        None => return Html("Not logged in".to_string()).into_response(),
    };

    let progression_data = sqlx::query!(
        r#"SELECT
            cs.completed_at as date,
            cs.weight as "weight: f32",
            cs.reps as "reps: i32",
            cs.set_number as "set_number: i32",
            cs.exercise_id
           FROM completed_sets cs
           INNER JOIN active_workouts aw ON cs.active_workout_id = aw.id
           WHERE cs.exercise_id = ? AND aw.user_id = ?
           ORDER BY cs.completed_at DESC
           LIMIT 50"#,
        exercise_id,
        user_id
    )
    .fetch_all(&database_pool)
    .await
    .unwrap_or(Vec::new());

    let progression_data_vec: Vec<ProgressionDataPoint> = progression_data
        .into_iter()
        .map(|row| ProgressionDataPoint {
            date: row.date,
            weight: row.weight,
            reps: row.reps,
            volume: row.weight.map(|w| w * row.reps as f32),
            set_number: row.set_number,
        })
        .collect();

    // Serialize to JSON for JavaScript consumption
    let progression_data_json = serde_json::to_string(&progression_data_vec)
        .unwrap_or_else(|_| "[]".to_string());

    let template = ExerciseProgressionTemplate {
        exercise,
        progression_data: progression_data_vec,
        progression_data_json,
        current_user,
        is_dashboard: false,
    };

    Html(template.render().unwrap()).into_response()
}

pub fn router() -> Router<SqlitePool> {
    Router::new()
        .route("/exercises", get(list_exercises))
        .route("/exercises", post(create_exercise))
        .route("/exercises/{id}/progression", get(show_exercise_progression))
}
