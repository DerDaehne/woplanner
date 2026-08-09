use crate::error::AppError;
use crate::handlers::personal_records::check_and_update_prs;
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
    pub pr_notifications: Option<Vec<String>>,
    pub overload_suggestion: Option<String>,
}

async fn get_current_user(session: &Session, database_pool: &SqlitePool) -> Result<Option<User>, AppError> {
    if let Ok(Some(user_id)) = session.get::<String>("current_user_id").await {
        let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", user_id)
            .fetch_optional(database_pool)
            .await?;
        Ok(user)
    } else {
        Ok(None)
    }
}

async fn determine_current_exercise(
    database_pool: &SqlitePool,
    active_workout_id: &str,
    workout_id: &str,
) -> Result<Option<WorkoutExerciseDetail>, AppError> {
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
    .await?;

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
            return Ok(Some(exercise));
        }
    }
    Ok(None)
}

async fn calculate_progress_percent(
    database_pool: &SqlitePool,
    active_workout_id: &str,
    workout_id: &str,
) -> Result<f32, AppError> {
    let total_planned_sets = sqlx::query_scalar!(
        "SELECT COALESCE(SUM(target_sets), 0) FROM workout_exercises WHERE workout_id = ?",
        workout_id
    )
    .fetch_one(database_pool)
    .await? as f32;

    if total_planned_sets == 0.0 {
        return Ok(0.0);
    }

    let completed_sets_count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM completed_sets WHERE active_workout_id = ?",
        active_workout_id
    )
    .fetch_one(database_pool)
    .await? as f32;

    Ok((completed_sets_count / total_planned_sets * 100.0).min(100.0))
}

/// Generate progressive overload suggestion based on completed set
fn generate_overload_suggestion(completed_set: &CompletedSet, _current_weight: Option<f32>) -> Option<String> {
    // Only provide suggestions for weighted exercises
    let weight = match completed_set.weight {
        Some(w) => w,
        None => return None, // No suggestions for bodyweight exercises
    };

    if completed_set.reps > 12 {
        // User is doing too many reps - suggest increasing weight
        let suggested_weight = weight + 2.5; // Standard increment of 2.5kg
        Some(format!(
            "💪 Du schaffst mehr als 12 Wiederholungen! Versuche es beim nächsten Set mit {}kg für optimales Muskelwachstum.",
            suggested_weight
        ))
    } else if completed_set.reps < 6 {
        // User is struggling - suggest decreasing weight
        let suggested_weight = (weight - 2.5).max(0.0); // Don't go below 0
        if suggested_weight > 0.0 {
            Some(format!(
                "⚠️ Weniger als 6 Wiederholungen können auf zu hohes Gewicht hindeuten. Versuche es mit {}kg für bessere Form und Kontrolle.",
                suggested_weight
            ))
        } else {
            Some(
                "⚠️ Weniger als 6 Wiederholungen - versuche mit weniger Gewicht oder Bodyweight zu trainieren für bessere Form.".to_string()
            )
        }
    } else {
        // Reps are in the optimal range (6-12), no suggestion needed
        None
    }
}

pub async fn start_training(
    State(database_pool): State<SqlitePool>,
    session: Session,
    Form(form): Form<StartWorkoutForm>,
) -> Result<impl IntoResponse, AppError> {
    // Validate input
    if form.workout_id.is_empty() {
        return Err(AppError::BadRequest("Workout ID cannot be empty".to_string()));
    }

    let current_user = match get_current_user(&session, &database_pool).await? {
        Some(user) => user,
        None => {
            let mut headers = HeaderMap::new();
            headers.insert("HX-Redirect", HeaderValue::from_static("/users"));
            return Ok((headers, Html("Not logged in".to_string())).into_response());
        }
    };

    // Verify workout exists
    sqlx::query_as!(
        Workout,
        "SELECT id, user_id, name, description, is_active, schedule_type as \"schedule_type!: String\", schedule_day as \"schedule_day: i32\", created_at, updated_at FROM workouts WHERE id = ?",
        form.workout_id
    ).fetch_optional(&database_pool).await?
        .ok_or_else(|| AppError::NotFound(format!("Workout '{}' not found", form.workout_id)))?;

    let existing_active = sqlx::query_as!(
        ActiveWorkout,
        "SELECT * from active_workouts WHERE user_id = ? LIMIT 1",
        current_user.id
    )
    .fetch_optional(&database_pool)
    .await?;

    if let Some(active) = existing_active {
        let mut headers = HeaderMap::new();
        headers.insert(
            "HX-Redirect",
            HeaderValue::from_str(&format!("/live-training/{}", active.id))?,
        );
        return Ok((headers, Html("Redirecting to active training".to_string())).into_response());
    }

    let new_active = ActiveWorkout::new(current_user.id, form.workout_id);

    sqlx::query!(
        "INSERT INTO active_workouts (id, user_id, workout_id, started_at, created_at) VALUES (?, ?, ?, ?, ?)",
        new_active.id,
        new_active.user_id,
        new_active.workout_id,
        new_active.started_at,
        new_active.created_at
    ).execute(&database_pool).await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Redirect",
        HeaderValue::from_str(&format!("/live-training/{}", new_active.id))?,
    );
    Ok((headers, Html("Training started!".to_string())).into_response())
}

pub async fn show_live_training(
    Path(active_workout_id): Path<String>,
    State(database_pool): State<SqlitePool>,
    session: Session,
) -> Result<impl IntoResponse, AppError> {
    let current_user = get_current_user(&session, &database_pool).await?;

    let active_workout = sqlx::query_as!(
        ActiveWorkout,
        "SELECT * FROM active_workouts WHERE id = ?",
        active_workout_id
    )
    .fetch_optional(&database_pool)
    .await?;

    let active_workout = match active_workout {
        Some(workout) => workout,
        None => return Err(AppError::NotFound("Active workout not found".to_string())),
    };

    let workout = sqlx::query_as!(
        Workout,
        r#"SELECT
            id, user_id, name, description, is_active, schedule_type as "schedule_type!: String",
            schedule_day as "schedule_day: i32", created_at, updated_at FROM workouts WHERE id = ?"#,
        active_workout.workout_id
    ).fetch_one(&database_pool).await?;

    let current_exercise = determine_current_exercise(
        &database_pool,
        &active_workout_id,
        &active_workout.workout_id,
    )
    .await?;

    let progress_percent = calculate_progress_percent(
        &database_pool,
        &active_workout_id,
        &active_workout.workout_id,
    )
    .await?;

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
        .await?
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

    // Retrieve and clear PR notifications from session
    let pr_notifications: Option<Vec<String>> = session.get("pr_notifications").await.ok().flatten();
    if pr_notifications.is_some() {
        let _ = session.remove::<Vec<String>>("pr_notifications").await;
    }

    // Retrieve and clear overload suggestion from session
    let overload_suggestion: Option<String> = session.get("overload_suggestion").await.ok().flatten();
    if overload_suggestion.is_some() {
        let _ = session.remove::<String>("overload_suggestion").await;
    }

    let template = LiveTrainingTemplate {
        active_workout_view,
        current_exercise_sets,
        current_user,
        is_dashboard: false,
        pr_notifications,
        overload_suggestion,
    };

    Ok(Html(template.render()?).into_response())
}

pub async fn complete_set(
    Path(active_workout_id): Path<String>,
    State(database_pool): State<SqlitePool>,
    session: Session,
    Form(form): Form<CompleteSetForm>,
) -> Result<impl IntoResponse, AppError> {
    // Validate input
    if form.exercise_id.is_empty() {
        return Err(AppError::BadRequest("Exercise ID cannot be empty".to_string()));
    }
    if form.reps < 1 || form.reps > 100 {
        return Err(AppError::BadRequest("Reps must be between 1 and 100".to_string()));
    }
    if let Some(weight) = form.weight {
        if weight < 0.0 || weight > 10000.0 {
            return Err(AppError::BadRequest("Weight must be between 0 and 10000 kg".to_string()));
        }
    }

    // Get user_id from active workout
    let active_workout = sqlx::query_as!(
        ActiveWorkout,
        "SELECT * FROM active_workouts WHERE id = ?",
        active_workout_id
    )
    .fetch_optional(&database_pool)
    .await?;

    let active_workout = match active_workout {
        Some(workout) => workout,
        None => return Err(AppError::NotFound("Active workout not found".to_string())),
    };

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
        form.exercise_id.clone(),
        next_set_number,
        form.weight,
        form.reps,
    );
    // Leere Eingabe ist keine Notiz — sonst landet "" statt NULL in der DB
    // und das Template rendert ein einsames Paar Anführungszeichen.
    completed_set.notes = form.notes.filter(|n| !n.trim().is_empty());

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
    ).execute(&database_pool).await?;

    // Check for new personal records
    if let Ok(prs) = check_and_update_prs(
        &database_pool,
        &active_workout.user_id,
        &form.exercise_id,
        &completed_set,
    )
    .await
    {
        if !prs.is_empty() {
            // Store PR notifications in session for display
            let _ = session.insert("pr_notifications", prs).await;
        }
    }

    // Generate progressive overload suggestion
    if let Some(suggestion) = generate_overload_suggestion(&completed_set, form.weight) {
        let _ = session.insert("overload_suggestion", suggestion).await;
    }

    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Redirect",
        HeaderValue::from_str(&format!("/live-training/{}", active_workout_id))?,
    );
    Ok((headers, Html("Set completed".to_string())).into_response())
}

pub async fn finish_training(
    Path(active_workout_id): Path<String>,
    State(database_pool): State<SqlitePool>,
    Form(form): Form<FinishTrainingForm>,
) -> Result<impl IntoResponse, AppError> {
    let active_workout = sqlx::query_as!(
        ActiveWorkout,
        "SELECT * FROM active_workouts WHERE id = ?",
        active_workout_id
    )
    .fetch_optional(&database_pool)
    .await?;

    let active_workout = match active_workout {
        Some(workout) => workout,
        None => return Err(AppError::NotFound("Active workout not found".to_string())),
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

    // Leere Eingabe ist keine Notiz — siehe complete_set weiter oben.
    let notes = form.notes.filter(|n| !n.trim().is_empty());
    let completed_workout =
        CompletedWorkout::new(active_workout, total_sets, total_volume_kg, notes);

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
    ).execute(&database_pool).await?;

    sqlx::query!(
        "DELETE FROM active_workouts WHERE id = ?",
        active_workout_id
    )
    .execute(&database_pool)
    .await?;

    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", HeaderValue::from_static("/dashboard"));

    Ok((
        headers,
        Html("Training completed successfully!".to_string()),
    ).into_response())
}

pub fn router() -> Router<SqlitePool> {
    Router::new()
        .route("/start-training", post(start_training))
        .route("/live-training/{id}", get(show_live_training))
        .route("/live-training/{id}/complete-set", post(complete_set))
        .route("/live-training/{id}/finish", post(finish_training))
}
