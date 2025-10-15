use crate::models::{
    ActiveWorkout, ActiveWorkoutView, CompleteSetForm, CompletedSet, CompletedSetDetail,
    CompletedWorkout, FinishTrainingForm, StartWorkoutForm, User, Workout, WorkoutExerciseDetail,
};
use askama::Template;
use axum::{
    Form, Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue},
    response::{Html, IntoResponse},
    routing::{get, post},
};
use sqlx::SqlitePool;
use tower_sessions::Session;

#[derive(Template)]
#[template(path = "live_training/active.html")]
pub struct LiveTrainingTemplate {
    pub active_workout_view: ActiveWorkoutView,
    pub current_exercise_sets: Vec<CompletedSetDetail>,
    pub current_user: Option<User>,
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

async fn determine_current_exercise(
    database_pool: &SqlitePool,
    active_workout_id: &str,
    workout_id: &str,
) -> Option<WorkoutExerciseDetail> {
    let workout_exercise = sqlx::query_as!(
        WorkoutExerciseDetail,
        r#"SELECT
            we.position as "position: i32",
            we.target_sets as "target_sets: i32",
            we.target_weight as "target_weight: f32",
            we.notes,
            e.id as exercise_id,
            e.name as exercise_name,
            e.instructions as exercise_instructions,
            e.video_url as exercise_video_url
           FROM workout_exercises we
           INNER JOIN exercises e ON we.exercise_id = e.id
           WHERE we.workout_id = ?
           ORDER BY we.position ASC"#,
        workout_id
    )
    .fetch_all(database_pool)
    .await
    .unwrap_or(Vec::new());

    for exercise in workout_exercise {
        let completed_sets_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM completed_sets WHERE active_workout_id = ? AND exercise_id = ?",
            active_workout_id,
            exercise.exercise_id
        )
        .fetch_one(database_pool)
        .await
        .unwrap_or(0) as i32;

        if completed_sets_count < exercise.target_sets {
            return Some(exercise);
        }
    }
    None
}

async fn calculate_progress_percent(
    database_pool: &SqlitePool,
    active_workout_id: &str,
    workout_id: &str,
) -> f32 {
    let total_planned_sets = sqlx::query_scalar!(
        "SELECT COALESCE(SUM(target_sets), 0) FROM workout_exercises WHERE workout_id = ?",
        workout_id
    )
    .fetch_one(database_pool)
    .await
    .unwrap_or(0) as f32;

    if total_planned_sets == 0.0 {
        return 0.0;
    }

    let completed_sets_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM completed_sets WHERE active_workout_id = ?",
        active_workout_id
    )
    .fetch_one(database_pool)
    .await
    .unwrap_or(0) as f32;

    (completed_sets_count / total_planned_sets * 100.0).min(100.0)
}

pub async fn start_training(
    State(database_pool): State<SqlitePool>,
    session: Session,
    Form(form): Form<StartWorkoutForm>,
) -> impl IntoResponse {
    let current_user = match get_current_user(&session, &database_pool).await {
        Some(user) => user,
        None => {
            let mut headers = HeaderMap::new();
            headers.insert("HX-Redirect", HeaderValue::from_static("/users"));
            return (headers, Html("Not logged in".to_string())).into_response();
        }
    };

    let existing_active = sqlx::query_as!(
        ActiveWorkout,
        "SELECT * from active_workouts WHERE user_id = ? LIMIT 1",
        current_user.id
    )
    .fetch_optional(&database_pool)
    .await
    .unwrap_or(None);

    if let Some(active) = existing_active {
        let mut headers = HeaderMap::new();
        headers.insert(
            "HX-Redirect",
            HeaderValue::from_str(&format!("/live-training/{}", active.id)).unwrap(),
        );
        return (headers, Html("Redirecting to active training".to_string())).into_response();
    }

    let new_active = ActiveWorkout::new(current_user.id, form.workout_id);

    sqlx::query!(
    "INSERT INTO active_workouts (id, user_id, workout_id, started_at, created_at) VALUES (?, ?, ?, ?, ?)",
        new_active.id,
        new_active.user_id,
        new_active.workout_id,
        new_active.started_at,
        new_active.created_at
    )
    .execute(&database_pool)
    .await
    .expect("Failed to create active workout");

    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Redirect",
        HeaderValue::from_str(&format!("/live-training/{}", new_active.id)).unwrap(),
    );
    (headers, Html("Training started!".to_string())).into_response()
}

pub async fn show_live_training(
    Path(active_workout_id): Path<String>,
    State(database_pool): State<SqlitePool>,
    session: Session,
) -> impl IntoResponse {
    let current_user = get_current_user(&session, &database_pool).await;

    let active_workout = match sqlx::query_as!(
        ActiveWorkout,
        "SELECT * FROM active_workouts WHERE id = ?",
        active_workout_id
    )
    .fetch_optional(&database_pool)
    .await
    {
        Ok(Some(workout)) => workout,
        _ => return Html("Active workout not found".to_string()).into_response(),
    };

    let workout = sqlx::query_as!(
        Workout,
        r#"SELECT
            id, user_id, name, description, is_active, schedule_type as "schedule_type!: String",
            schedule_day as "schedule_day: i32", created_at, updated_at FROM workouts WHERE id = ?"#,
        active_workout.workout_id
    ).fetch_one(&database_pool).await.expect("Workout should exist");

    let current_exercise = determine_current_exercise(
        &database_pool,
        &active_workout_id,
        &active_workout.workout_id,
    )
    .await;

    let progress_percent = calculate_progress_percent(
        &database_pool,
        &active_workout_id,
        &active_workout.workout_id,
    )
    .await;

    let total_sets_completed = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM completed_sets WHERE active_workout_id = ?",
        active_workout_id
    )
    .fetch_one(&database_pool)
    .await
    .unwrap_or(0) as i32;

    let current_exercise_sets = if let Some(ref exercise) = current_exercise {
        sqlx::query_as!(
            CompletedSetDetail,
            r#"SELECT
                cs.id,
                cs.set_number as "set_number: i32",
                cs.weight as "weight: f32",
                cs.reps as "reps: i32",
                cs.notes,
                cs.completed_at,
                e.name as exercise_name,
                e.instructions as exercise_instructions
               FROM completed_sets cs
               INNER JOIN exercises e ON cs.exercise_id = e.id
               WHERE cs.active_workout_id = ? AND cs.exercise_id = ?
               ORDER BY cs.completed_at DESC"#,
            active_workout_id,
            exercise.exercise_id
        )
        .fetch_all(&database_pool)
        .await
        .unwrap_or(Vec::new())
    } else {
        Vec::new()
    };

    let active_workout_view = ActiveWorkoutView {
        active_workout,
        workout_name: workout.name,
        total_sets_completed,
        current_exercise,
        progress_percent,
    };

    let template = LiveTrainingTemplate {
        active_workout_view,
        current_exercise_sets,
        current_user,
        is_dashboard: false,
    };

    Html(template.render().unwrap()).into_response()
}

pub async fn complete_set(
    Path(active_workout_id): Path<String>,
    State(database_pool): State<SqlitePool>,
    Form(form): Form<CompleteSetForm>,
) -> impl IntoResponse {
    let next_set_number = sqlx::query_scalar!(
        "SELECT COALESCE(MAX(set_number), 0) +1
         FROM completed_sets
         WHERE active_workout_id = ? AND exercise_id = ?",
        active_workout_id,
        form.exercise_id
    )
    .fetch_one(&database_pool)
    .await
    .unwrap_or(1) as i32;

    let mut completed_set = CompletedSet::new(
        active_workout_id.clone(),
        form.exercise_id,
        next_set_number,
        form.weight,
        form.reps,
    );
    completed_set.notes = form.notes;

    sqlx::query!(
        "INSERT INTO completed_sets (id, active_workout_id, exercise_id, set_number, weight, reps, notes, completed_at, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        completed_set.id,
        completed_set.active_workout_id,
        completed_set.exercise_id,
        completed_set.set_number,
        completed_set.weight,
        completed_set.reps,
        completed_set.notes,
        completed_set.completed_at,
        completed_set.created_at
    ).execute(&database_pool).await.expect("Failed to save completed set");

    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Redirect",
        HeaderValue::from_str(&format!("/live-training/{}", active_workout_id)).unwrap(),
    );
    (headers, Html("Set completed".to_string())).into_response()
}

pub async fn finish_training(
    Path(active_workout_id): Path<String>,
    State(database_pool): State<SqlitePool>,
    Form(form): Form<FinishTrainingForm>,
) -> impl IntoResponse {
    let active_workout = match sqlx::query_as!(
        ActiveWorkout,
        "SELECT * FROM active_workouts WHERE id = ?",
        active_workout_id
    )
    .fetch_optional(&database_pool)
    .await
    {
        Ok(Some(workout)) => workout,
        _ => return Html("Active workout not found".to_string()).into_response(),
    };

    let total_sets = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM completed_sets WHERE active_workout_id = ?",
        active_workout_id
    )
    .fetch_one(&database_pool)
    .await
    .unwrap_or(0) as i32;

    let total_volume_kg = sqlx::query_scalar!(
        r#"SELECT COALESCE(SUM(weight * reps), 0.0) as "total_volume!: f32"
            FROM completed_sets
            WHERE active_workout_id = ? AND weight IS NOT NULL"#,
        active_workout_id
    )
    .fetch_one(&database_pool)
    .await
    .unwrap_or(0.0);

    let completed_workout =
        CompletedWorkout::new(active_workout, total_sets, total_volume_kg, form.notes);

    sqlx::query!(
    r#"INSERT INTO completed_workouts
        (id, user_id, workout_id, started_at, completed_at, total_duration_minutes, total_sets, total_volume_kg, notes, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        completed_workout.id,
        completed_workout.user_id,
        completed_workout.workout_id,
        completed_workout.started_at,
        completed_workout.completed_at,
        completed_workout.total_duration_minutes,
        completed_workout.total_sets,
        completed_workout.total_volume_kg,
        completed_workout.notes,
        completed_workout.created_at
    ).execute(&database_pool).await.expect("Failed to save completed workout!");

    sqlx::query!(
        "DELETE FROM active_workouts WHERE id = ?",
        active_workout_id
    )
    .execute(&database_pool)
    .await
    .expect("Failed to delete active workout");

    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", HeaderValue::from_static("/dashboard"));

    (
        headers,
        Html("Training completed successfully!".to_string()),
    )
        .into_response()
}

pub fn router() -> Router<SqlitePool> {
    Router::new()
        .route("/start-training", post(start_training))
        .route("/live-training/{id}", get(show_live_training))
        .route("/live-training/{id}/complete-set", post(complete_set))
        .route("/live-training/{id}/finish", post(finish_training))
}
