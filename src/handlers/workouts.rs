use crate::error::AppError;
use crate::models::{AddExerciseToWorkoutForm, CreateWorkoutForm, UpdateWorkoutScheduleForm};
use crate::models::{Exercise, User, Workout, WorkoutExercise, WorkoutExerciseDetail};
use crate::workout_yaml::{PlanExercise, WorkoutPlan, filename_for, from_yaml, to_yaml};
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

pub async fn list_workouts(
    State(database_pool): State<SqlitePool>,
    session: Session,
) -> Result<impl IntoResponse, AppError> {
    let current_user = get_current_user(&session, &database_pool).await?;

    let current_user = match current_user {
        Some(user) => user,
        None => {
            let mut headers = HeaderMap::new();
            headers.insert("HX-Redirect", HeaderValue::from_static("/users"));
            return Ok((headers, Html("Not logged in".to_string())).into_response());
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
    .await?;

    let template = WorkoutListTemplate {
        workouts,
        current_user: Some(current_user),
        is_dashboard: false,
    };

    Ok(Html(template.render()?).into_response())
}

pub async fn show_workout(
    Path(workout_id): Path<String>,
    State(database_pool): State<SqlitePool>,
    session: Session,
) -> Result<impl IntoResponse, AppError> {
    let current_user = get_current_user(&session, &database_pool).await?;

    let workout = sqlx::query_as!(
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
    .await?;

    let workout = match workout {
        Some(w) => w,
        None => return Err(AppError::NotFound(format!("Workout '{}' not found", workout_id))),
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
            e.instructions as exercise_instructions,
            e.video_url as exercise_video_url
        FROM workout_exercises we INNER JOIN exercises e ON we.exercise_id = e.id WHERE we.workout_id = ? ORDER BY we.position ASC"#,
        workout_id
    ).fetch_all(&database_pool).await?;

    let available_exercises = sqlx::query_as!(Exercise, "SELECT id, name, instructions, video_url, created_at FROM exercises ORDER BY name")
        .fetch_all(&database_pool)
        .await?;

    let template = WorkoutDetailTemplate {
        workout,
        exercises,
        available_exercises,
        current_user,
        is_dashboard: false,
    };

    Ok(Html(template.render()?).into_response())
}

/// Plan als YAML zum Herunterladen. Format: Notiz `concept-workout-yaml`.
pub async fn export_workout(
    Path(workout_id): Path<String>,
    State(database_pool): State<SqlitePool>,
    session: Session,
) -> Result<impl IntoResponse, AppError> {
    let current_user = match get_current_user(&session, &database_pool).await? {
        Some(user) => user,
        None => return Err(AppError::Unauthorized),
    };

    let workout = sqlx::query_as!(
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
    .await?;

    // Fremde Pläne verhalten sich wie nicht vorhandene — die Existenz eines
    // fremden Plans ist selbst schon eine Auskunft.
    let workout = match workout {
        Some(w) if w.user_id == current_user.id => w,
        _ => return Err(AppError::NotFound(format!("Workout '{}' not found", workout_id))),
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
            e.instructions as exercise_instructions,
            e.video_url as exercise_video_url
        FROM workout_exercises we INNER JOIN exercises e ON we.exercise_id = e.id WHERE we.workout_id = ? ORDER BY we.position ASC"#,
        workout_id
    ).fetch_all(&database_pool).await?;

    let plan = WorkoutPlan {
        name: workout.name.clone(),
        description: workout.description.clone(),
        schedule_type: workout.schedule_type.clone(),
        schedule_day: workout.schedule_day.map(|d| d as i64),
        exercises: exercises
            .into_iter()
            .map(|e| PlanExercise {
                name: e.exercise_name,
                instructions: Some(e.exercise_instructions),
                video_url: e.exercise_video_url,
                sets: e.target_sets as i64,
                weight: e.target_weight.map(|w| w as f64),
                notes: e.notes,
            })
            .collect(),
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/yaml; charset=utf-8"),
    );
    headers.insert(
        "Content-Disposition",
        HeaderValue::from_str(&format!(
            "attachment; filename=\"{}\"",
            filename_for(&workout.name)
        ))
        .unwrap_or(HeaderValue::from_static("attachment; filename=\"workout.yaml\"")),
    );

    Ok((headers, to_yaml(&plan)).into_response())
}

pub async fn create_workout(
    State(database_pool): State<SqlitePool>,
    session: Session,
    Form(form): Form<CreateWorkoutForm>,
) -> Result<impl IntoResponse, AppError> {
    let current_user = match get_current_user(&session, &database_pool).await? {
        Some(user) => user,
        None => return Err(AppError::Unauthorized),
    };

    // Validate input
    let name = form.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::BadRequest("Workout name cannot be empty".to_string()));
    }
    if name.len() > 100 {
        return Err(AppError::BadRequest("Workout name must be 100 characters or less".to_string()));
    }

    let new_workout = Workout::new(current_user.id.clone(), name, form.description);

    sqlx::query!(
        "INSERT INTO workouts (id, user_id, name, description, is_active, schedule_type, schedule_day, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        new_workout.id,
        new_workout.user_id,
        new_workout.name,
        new_workout.description,
        new_workout.is_active,
        new_workout.schedule_type,
        new_workout.schedule_day,
        new_workout.created_at,
        new_workout.updated_at
    ).execute(&database_pool).await?;

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
    .await?;

    let template = WorkoutListPartialTemplate { workouts };
    Ok(Html(template.render()?).into_response())
}

#[derive(serde::Deserialize)]
pub struct ImportWorkoutForm {
    pub yaml: String,
}

/// Plan aus YAML anlegen. Format: Notiz `concept-workout-yaml`.
///
/// Erst vollständig prüfen, dann in einer Transaktion schreiben — ein
/// abgebrochener Import darf keinen halben Plan hinterlassen.
pub async fn import_workout(
    State(database_pool): State<SqlitePool>,
    session: Session,
    Form(form): Form<ImportWorkoutForm>,
) -> Result<impl IntoResponse, AppError> {
    let current_user = match get_current_user(&session, &database_pool).await? {
        Some(user) => user,
        None => return Err(AppError::Unauthorized),
    };

    let plan = from_yaml(&form.yaml).map_err(|e| AppError::BadRequest(e.to_string()))?;

    let mut tx = database_pool.begin().await?;
    let workout = write_plan(&mut tx, &current_user.id, &plan).await?;
    tx.commit().await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Redirect",
        HeaderValue::from_str(&format!("/workouts/{}", workout.id))
            .unwrap_or(HeaderValue::from_static("/workouts")),
    );
    Ok((headers, Html(format!("Imported '{}'", workout.name))).into_response())
}

/// Den geprüften Plan schreiben. Nimmt eine Verbindung statt des Pools, damit
/// der Aufrufer die Transaktion besitzt — nur so lässt sich prüfen, dass ein
/// Fehler nichts hinterlässt.
pub async fn write_plan(
    conn: &mut sqlx::SqliteConnection,
    user_id: &str,
    plan: &crate::workout_yaml::WorkoutPlan,
) -> Result<Workout, AppError> {
    let mut workout = Workout::new(
        user_id.to_string(),
        plan.name.clone(),
        plan.description.clone(),
    );
    workout.schedule_type = plan.schedule_type.clone();
    workout.schedule_day = plan.schedule_day.map(|d| d as i32);

    sqlx::query!(
        "INSERT INTO workouts (id, user_id, name, description, is_active, schedule_type, schedule_day, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        workout.id,
        workout.user_id,
        workout.name,
        workout.description,
        workout.is_active,
        workout.schedule_type,
        workout.schedule_day,
        workout.created_at,
        workout.updated_at
    )
    .execute(&mut *conn)
    .await?;

    for (index, planned) in plan.exercises.iter().enumerate() {
        let position = index as i32 + 1;

        // Zuordnung über den Namen, ohne Groß-/Kleinschreibung. Ein Modell
        // kennt keine IDs — der Name ist das einzige stabile Merkmal.
        let existing = sqlx::query_scalar!(
            "SELECT id FROM exercises WHERE lower(trim(name)) = lower(trim(?)) LIMIT 1",
            planned.name
        )
        .fetch_optional(&mut *conn)
        .await?;

        let exercise_id = match existing {
            // Vorhandene Übung bleibt unverändert: ein Import ist kein Weg,
            // fremde Anleitungen in die eigene Bibliothek zu schreiben.
            Some(id) => id,
            None => {
                let instructions = planned
                    .instructions
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| {
                        AppError::BadRequest(format!(
                            "exercise {position} '{}': unknown exercise, so 'instructions' is required",
                            planned.name
                        ))
                    })?;
                let new_exercise = Exercise::new(
                    planned.name.clone(),
                    instructions.to_string(),
                    planned.video_url.clone(),
                );
                sqlx::query!(
                    "INSERT INTO exercises (id, name, instructions, video_url, created_at) VALUES (?, ?, ?, ?, ?)",
                    new_exercise.id,
                    new_exercise.name,
                    new_exercise.instructions,
                    new_exercise.video_url,
                    new_exercise.created_at
                )
                .execute(&mut *conn)
                .await?;
                new_exercise.id
            }
        };

        let mut link = WorkoutExercise::new(
            workout.id.clone(),
            exercise_id,
            position,
            planned.sets as i32,
            planned.weight.map(|w| w as f32),
        );
        link.notes = planned.notes.clone();

        sqlx::query!(
            "INSERT INTO workout_exercises (id, workout_id, exercise_id, position, target_sets, target_weight, notes, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            link.id,
            link.workout_id,
            link.exercise_id,
            link.position,
            link.target_sets,
            link.target_weight,
            link.notes,
            link.created_at
        )
        .execute(&mut *conn)
        .await?;
    }

    Ok(workout)
}

pub async fn add_exercise_to_workout(
    Path(workout_id): Path<String>,
    State(database_pool): State<SqlitePool>,
    Form(form): Form<AddExerciseToWorkoutForm>,
) -> Result<impl IntoResponse, AppError> {
    // Validate input
    if form.exercise_id.is_empty() {
        return Err(AppError::BadRequest("Exercise ID cannot be empty".to_string()));
    }
    if form.target_sets < 1 || form.target_sets > 50 {
        return Err(AppError::BadRequest("Target sets must be between 1 and 50".to_string()));
    }

    // Verify workout exists first
    sqlx::query_as!(
        Workout,
        "SELECT id, user_id, name, description, is_active, schedule_type as \"schedule_type!: String\", schedule_day as \"schedule_day: i32\", created_at, updated_at FROM workouts WHERE id = ?",
        workout_id
    ).fetch_optional(&database_pool).await?
        .ok_or_else(|| AppError::NotFound(format!("Workout '{}' not found", workout_id)))?;

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

    sqlx::query!(
        "INSERT INTO workout_exercises (id, workout_id, exercise_id, position, target_sets, target_weight, notes, created_at) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)", 
        exercise_with_notes.id,
        exercise_with_notes.workout_id,
        exercise_with_notes.exercise_id,
        exercise_with_notes.position,
        exercise_with_notes.target_sets,
        exercise_with_notes.target_weight,
        exercise_with_notes.notes,
        exercise_with_notes.created_at
    ).execute(&database_pool).await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Redirect",
        HeaderValue::from_str(&format!("/workouts/{}", workout_id))?,
    );
    Ok((headers, Html("Exercise added".to_string())).into_response())
}

pub async fn update_workout_schedule(
    Path(workout_id): Path<String>,
    State(database_pool): State<SqlitePool>,
    Form(form): Form<UpdateWorkoutScheduleForm>,
) -> Result<impl IntoResponse, AppError> {
    // Verify workout exists
    sqlx::query_as!(
        Workout,
        "SELECT id, user_id, name, description, is_active, schedule_type as \"schedule_type!: String\", schedule_day as \"schedule_day: i32\", created_at, updated_at FROM workouts WHERE id = ?",
        workout_id
    ).fetch_optional(&database_pool).await?
        .ok_or_else(|| AppError::NotFound(format!("Workout '{}' not found", workout_id)))?;

    let updated_at = chrono::Utc::now().to_rfc3339();
    sqlx::query!(
        "UPDATE workouts SET schedule_type = ?, schedule_day = ?, updated_at = ? WHERE id = ?",
        form.schedule_type,
        form.schedule_day,
        updated_at,
        workout_id
    )
    .execute(&database_pool)
    .await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Redirect",
        HeaderValue::from_str(&format!("/workouts/{}", workout_id))?,
    );
    Ok((headers, Html("Schedule updated".to_string())).into_response())
}

pub async fn toggle_workout_active(
    Path(workout_id): Path<String>,
    State(database_pool): State<SqlitePool>,
) -> Result<impl IntoResponse, AppError> {
    let current_status =
        sqlx::query_scalar!("SELECT is_active FROM workouts WHERE id = ?", workout_id)
            .fetch_one(&database_pool)
            .await?;

    let new_status = !current_status.unwrap_or(true);
    let updated_at = chrono::Utc::now().to_rfc3339();
    sqlx::query!(
        "UPDATE workouts SET is_active = ?, updated_at = ? WHERE id = ?",
        new_status,
        updated_at,
        workout_id
    )
    .execute(&database_pool)
    .await?;

    let mut headers = HeaderMap::new();
    headers.insert(
        "HX-Redirect",
        HeaderValue::from_str(&format!("/workouts/{}", workout_id))?,
    );
    Ok((headers, Html("Status updated".to_string())).into_response())
}

pub fn router() -> Router<SqlitePool> {
    Router::new()
        .route("/workouts", get(list_workouts))
        .route("/workouts", post(create_workout))
        .route("/workouts/import", post(import_workout))
        .route("/workouts/{id}", get(show_workout))
        .route("/workouts/{id}/export", get(export_workout))
        .route("/workouts/{id}/exercises", post(add_exercise_to_workout))
        .route("/workouts/{id}/schedule", post(update_workout_schedule))
        .route("/workouts/{id}/toggle", post(toggle_workout_active))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workout_yaml::{PlanExercise, WorkoutPlan};

    fn plan_with_unknown_exercise_last() -> WorkoutPlan {
        WorkoutPlan {
            name: "Kaputter Plan".into(),
            description: None,
            schedule_type: "manual".into(),
            schedule_day: None,
            exercises: vec![
                PlanExercise {
                    name: "Neue Übung mit Anleitung".into(),
                    instructions: Some("Sauber ausführen.".into()),
                    video_url: None,
                    sets: 3,
                    weight: None,
                    notes: None,
                },
                // Unbekannt und ohne Anleitung: bricht ab, nachdem oben
                // bereits Workout, Übung und Zuordnung geschrieben wurden.
                PlanExercise {
                    name: "Unbekannt ohne Anleitung".into(),
                    instructions: None,
                    video_url: None,
                    sets: 3,
                    weight: None,
                    notes: None,
                },
            ],
        }
    }

    async fn counts(pool: &SqlitePool) -> (i32, i32, i32) {
        let workouts = sqlx::query_scalar!(r#"SELECT COUNT(*) as "c!: i32" FROM workouts"#)
            .fetch_one(pool).await.unwrap();
        let exercises = sqlx::query_scalar!(r#"SELECT COUNT(*) as "c!: i32" FROM exercises"#)
            .fetch_one(pool).await.unwrap();
        let links = sqlx::query_scalar!(r#"SELECT COUNT(*) as "c!: i32" FROM workout_exercises"#)
            .fetch_one(pool).await.unwrap();
        (workouts, exercises, links)
    }

    async fn seed_user(pool: &SqlitePool) -> String {
        let id = "test-user".to_string();
        sqlx::query!(
            "INSERT INTO users (id, name, created_at) VALUES (?, ?, ?)",
            id, "Test", "2026-01-01T00:00:00Z"
        ).execute(pool).await.unwrap();
        id
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn failed_import_leaves_nothing_behind(pool: SqlitePool) {
        let user_id = seed_user(&pool).await;
        let before = counts(&pool).await;

        let mut tx = pool.begin().await.unwrap();
        let result = write_plan(&mut tx, &user_id, &plan_with_unknown_exercise_last()).await;
        assert!(result.is_err(), "plan with an unknown exercise must not import");
        // Kein commit: die Transaktion fällt beim Verwerfen zurück.
        drop(tx);

        assert_eq!(counts(&pool).await, before, "a failed import must write nothing");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn successful_import_writes_plan_exercises_and_order(pool: SqlitePool) {
        let user_id = seed_user(&pool).await;
        let mut plan = plan_with_unknown_exercise_last();
        plan.exercises.pop();
        plan.exercises.push(PlanExercise {
            name: "Zweite Übung".into(),
            instructions: Some("Auch sauber.".into()),
            video_url: None,
            sets: 5,
            weight: Some(60.0),
            notes: Some("langsam".into()),
        });

        let mut tx = pool.begin().await.unwrap();
        let workout = write_plan(&mut tx, &user_id, &plan).await.unwrap();
        tx.commit().await.unwrap();

        let (workouts, exercises, links) = counts(&pool).await;
        assert_eq!((workouts, exercises, links), (1, 2, 2));

        let positions = sqlx::query!(
            r#"SELECT we.position as "position: i32", e.name
               FROM workout_exercises we JOIN exercises e ON e.id = we.exercise_id
               WHERE we.workout_id = ? ORDER BY we.position"#,
            workout.id
        ).fetch_all(&pool).await.unwrap();

        assert_eq!(positions[0].position, 1);
        assert_eq!(positions[0].name, "Neue Übung mit Anleitung");
        assert_eq!(positions[1].position, 2);
        assert_eq!(positions[1].name, "Zweite Übung");
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn known_exercise_is_reused_and_not_overwritten(pool: SqlitePool) {
        let user_id = seed_user(&pool).await;
        sqlx::query!(
            "INSERT INTO exercises (id, name, instructions, video_url, created_at) VALUES (?, ?, ?, ?, ?)",
            "ex-1", "Bench Press", "Originalanleitung", None::<String>, "2026-01-01T00:00:00Z"
        ).execute(&pool).await.unwrap();

        let plan = WorkoutPlan {
            name: "Plan".into(),
            description: None,
            schedule_type: "manual".into(),
            schedule_day: None,
            exercises: vec![PlanExercise {
                // andere Schreibweise, zusätzliche Leerzeichen
                name: "  bench press  ".into(),
                instructions: Some("Fremde Anleitung".into()),
                video_url: Some("https://example.com".into()),
                sets: 3,
                weight: None,
                notes: None,
            }],
        };

        let mut tx = pool.begin().await.unwrap();
        write_plan(&mut tx, &user_id, &plan).await.unwrap();
        tx.commit().await.unwrap();

        let row = sqlx::query!(r#"SELECT COUNT(*) as "c!: i32" FROM exercises"#)
            .fetch_one(&pool).await.unwrap();
        assert_eq!(row.c, 1, "a known exercise must not be duplicated");

        let instructions = sqlx::query_scalar!("SELECT instructions FROM exercises WHERE id = 'ex-1'")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(instructions, "Originalanleitung", "import must not overwrite the library");
    }
}
