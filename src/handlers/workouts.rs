use crate::models::{AddExerciseToWorkoutForm, CreateWorkoutForm, UpdateWorkoutScheduleForm};
use crate::models::{Exercise, User, Workout, WorkoutExercise, WorkoutExerciseDetail};
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
#[template(path = "workouts/list.html")]
pub struct WorkoutListTemplate {
    pub workouts: Vec<Workout>,
    pub current_user: Option<User>,
    pub is_dashboard: bool,
}

#[derive(Template)]
#[template(path = "workouts/detail.html")]
pub struct WorkoutDetailTemplate {
    pub workout: Workout,
    pub exercises: Vec<WorkoutExerciseDetail>,
    pub available_exercises: Vec<Exercise>,
    pub current_user: Option<User>,
    pub is_dashboard: bool,
}

#[derive(Template)]
#[template(path = "workouts/workout_list_partial.html")]
pub struct WorkoutListPartialTemplate {
    pub workouts: Vec<Workout>,
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

pub async fn list_workouts(
    State(database_pool): State<SqlitePool>,
    session: Session,
) -> impl IntoResponse {
    let current_user = get_current_user(&session, &database_pool).await;

    let current_user = match current_user {
        Some(user) => user,
        None => {
            let mut headers = HeaderMap::new();
            headers.insert("HX-Redirect", HeaderValue::from_static("/users"));
            return (headers, Html("Not logged in".to_string())).into_response();
        }
    };

    let workouts = sqlx::query_as!(
        Workout,
        r#"SELECT 
            id,
            user_id,
            name,
            description,
            is_active,
            schedule_type as "schedule_type!: String",
            schedule_day as "schedule_day: i32",
            created_at,
            updated_at
            FROM workouts WHERE user_id = ? ORDER BY
            CASE WHEN schedule_type = 'rotation' THEN 0
                WHEN schedule_type = 'weekly' THEN 1
                WHEN schedule_type = 'manual' THEN 3
                ELSE 3 END,
            name ASC"#,
        current_user.id
    )
    .fetch_all(&database_pool)
    .await
    .unwrap_or(Vec::new());

    let template = WorkoutListTemplate {
        workouts,
        current_user: Some(current_user),
        is_dashboard: false,
    };

    Html(template.render().unwrap()).into_response()
}

pub async fn show_workout(
    Path(workout_id): Path<String>,
    State(database_pool): State<SqlitePool>,
    session: Session,
) -> impl IntoResponse {
    let current_user = get_current_user(&session, &database_pool).await;

    let workout = match sqlx::query_as!(
        Workout,
        r#"
        SELECT  
        id,
        user_id,
        name,
        description,
        is_active,
        schedule_type as "schedule_type!: String", 
        schedule_day as "schedule_day: i32",
        created_at,
        updated_at
        FROM workouts WHERE id = ?"#,
        workout_id
    )
    .fetch_optional(&database_pool)
    .await
    {
        Ok(Some(workout)) => workout,
        _ => {
            return Html("Workout not found".to_string()).into_response();
        }
    };

    let exercises = sqlx::query_as!(
    WorkoutExerciseDetail,
        r#"
        SELECT
            we.position as "position: i32",
            we.target_sets as "target_sets: i32",
            we.target_weight as "target_weight: f32",
            we.notes,
            e.id as exercise_id,
            e.name as exercise_name,
            e.instructions as exercise_instructions
        FROM workout_exercises we INNER JOIN exercises e ON we.exercise_id = e.id WHERE we.workout_id = ? ORDER BY we.position ASC "#, workout_id)
    .fetch_all(&database_pool).await.unwrap_or(Vec::new());

    let available_exercises = sqlx::query_as!(Exercise, "SELECT * FROM exercises ORDER BY name")
        .fetch_all(&database_pool)
        .await
        .unwrap_or(Vec::new());

    let template = WorkoutDetailTemplate {
        workout,
        exercises,
        available_exercises,
        current_user,
        is_dashboard: false,
    };

    Html(template.render().unwrap()).into_response()
}

pub async fn create_workout(
    State(database_pool): State<SqlitePool>,
    session: Session,
    Form(form): Form<CreateWorkoutForm>,
) -> impl IntoResponse {
    let current_user = match get_current_user(&session, &database_pool).await {
        Some(user) => user,
        None => {
            return Html("Not logged in".to_string()).into_response();
        }
    };

    let new_workout = Workout::new(current_user.id.clone(), form.name, form.description);

    sqlx::query!("INSERT INTO workouts (id, user_id, name, description, is_active, schedule_type,  schedule_day, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    new_workout.id,
    new_workout.user_id,
    new_workout.name,
    new_workout.description,
    new_workout.is_active,
    new_workout.schedule_type,
    new_workout.schedule_day,
    new_workout.created_at,
    new_workout.updated_at).execute(&database_pool).await.expect("Failed to create workout");

    let workouts = sqlx::query_as!(
        Workout,
        r#"
        SELECT 
        id,
        user_id,
        name,
        description,
        is_active,
        schedule_type as "schedule_type!: String",
        schedule_day as "schedule_day: i32",
        created_at,
        updated_at
        FROM workouts WHERE user_id = ? ORDER BY created_at DESC"#,
        current_user.id
    )
    .fetch_all(&database_pool)
    .await
    .unwrap_or(Vec::new());

    let template = WorkoutListPartialTemplate { workouts };
    Html(template.render().unwrap()).into_response()
}

pub async fn add_exercise_to_workout(
    Path(workout_id): Path<String>,
    State(database_pool): State<SqlitePool>,
    Form(form): Form<AddExerciseToWorkoutForm>,
) -> impl IntoResponse {
    let max_position = sqlx::query_scalar!(
        "SELECT COALESCE(MAX(position), 0) FROM workout_exercises WHERE workout_id = ?",
        workout_id
    )
    .fetch_one(&database_pool)
    .await
    .unwrap_or(0) as i32;

    let new_exercise = WorkoutExercise::new(
        workout_id.clone(),
        form.exercise_id,
        max_position + 1,
        form.target_sets,
        form.target_weight,
    );

    let mut exercise_with_notes = new_exercise.clone();
    exercise_with_notes.notes = form.notes;

    sqlx::query!("INSERT INTO workout_exercises (id, workout_id, exercise_id, position, target_sets, target_weight, notes, created_at) 
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)", 
        exercise_with_notes.id,
        exercise_with_notes.workout_id,
        exercise_with_notes.exercise_id,
        exercise_with_notes.position,
        exercise_with_notes.target_sets,
        exercise_with_notes.target_weight,
        exercise_with_notes.notes,
        exercise_with_notes.created_at).execute(&database_pool).await.expect("Failed to add exercise");

    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Redirect",
        HeaderValue::from_str(&format!("/workouts/{}", workout_id)).unwrap(),
    );
    (headers, Html("Exercise added".to_string())).into_response()
}

pub async fn update_workout_schedule(
    Path(workout_id): Path<String>,
    State(database_pool): State<SqlitePool>,
    Form(form): Form<UpdateWorkoutScheduleForm>,
) -> impl IntoResponse {
    let updated_at = chrono::Utc::now().to_rfc3339();
    sqlx::query!(
        "UPDATE workouts SET schedule_type = ?, schedule_day = ?, updated_at = ? WHERE id = ?",
        form.schedule_type,
        form.schedule_day,
        updated_at,
        workout_id
    )
    .execute(&database_pool)
    .await
    .expect("Failed to update schedule");

    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Redirect",
        HeaderValue::from_str(&format!("/workouts/{}", workout_id)).unwrap(),
    );
    (headers, Html("Schedule updated".to_string())).into_response()
}

pub async fn toggle_workout_active(
    Path(workout_id): Path<String>,
    State(database_pool): State<SqlitePool>,
) -> impl IntoResponse {
    let current_status =
        sqlx::query_scalar!("SELECT is_active FROM workouts WHERE id = ?", workout_id)
            .fetch_one(&database_pool)
            .await
            .unwrap_or(Some(true));

    let new_status = !current_status.unwrap_or(true);
    let updated_at = chrono::Utc::now().to_rfc3339();

    sqlx::query!(
        "UPDATE workouts SET is_active = ?, updated_at = ? WHERE id = ?",
        new_status,
        updated_at,
        workout_id
    )
    .execute(&database_pool)
    .await
    .expect("Failed to toggle active status");

    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Redirect",
        HeaderValue::from_str(&format!("/workouts/{}", workout_id)).unwrap(),
    );
    (headers, Html("Status updated".to_string())).into_response()
}

pub fn router() -> Router<SqlitePool> {
    Router::new()
        .route("/workouts", get(list_workouts))
        .route("/workouts", post(create_workout))
        .route("/workouts/{id}", get(show_workout))
        .route("/workouts/{id}/exercises", post(add_exercise_to_workout))
        .route("/workouts/{id}/schedule", post(update_workout_schedule))
        .route("/workouts/{id}/toggle", post(toggle_workout_active))
}
