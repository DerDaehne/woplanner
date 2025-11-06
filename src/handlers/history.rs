use crate::models::User;
use askama::Template;
use axum::{
    Router,
    extract::{Path, State},
    response::{Html, IntoResponse},
    routing::get,
};
use sqlx::SqlitePool;
use tower_sessions::Session;

#[derive(Template)]
#[template(path = "history/list.html")]
pub struct HistoryListTemplate {
    pub user: User,
    pub current_user: Option<User>,
    pub workouts: Vec<CompletedWorkoutWithName>,
    pub is_dashboard: bool,
}

#[derive(Template)]
#[template(path = "history/detail.html")]
pub struct HistoryDetailTemplate {
    pub user: User,
    pub current_user: Option<User>,
    pub workout: CompletedWorkoutWithName,
    pub exercises: Vec<ExerciseWithSets>,
    pub is_dashboard: bool,
}

#[derive(Debug, Clone)]
pub struct CompletedWorkoutWithName {
    pub id: String,
    pub user_id: String,
    pub workout_id: String,
    pub workout_name: String,
    pub started_at: String,
    pub completed_at: String,
    pub total_duration_minutes: i32,
    pub total_sets: i32,
    pub total_volume_kg: f32,
    pub notes: Option<String>,
    pub created_at: String,
}

impl CompletedWorkoutWithName {
    pub fn duration_display(&self) -> String {
        let mins = self.total_duration_minutes;
        if mins < 60 {
            format!("{}m", mins)
        } else {
            format!("{}h {}m", mins / 60, mins % 60)
        }
    }

    pub fn completed_date_display(&self) -> String {
        match chrono::DateTime::parse_from_rfc3339(&self.completed_at) {
            Ok(completed) => {
                let now = chrono::Utc::now();
                let diff = now.signed_duration_since(completed).num_days();

                match diff {
                    0 => "Today".to_string(),
                    1 => "Yesterday".to_string(),
                    n if n < 7 => format!("{} days ago", n),
                    _ => completed.format("%b %d, %Y").to_string(),
                }
            }
            Err(_) => "Unknown".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExerciseWithSets {
    pub exercise_name: String,
    pub sets: Vec<SetDetail>,
}

#[derive(Debug, Clone)]
pub struct SetDetail {
    pub set_number: i32,
    pub weight: Option<f32>,
    pub reps: i32,
    pub notes: Option<String>,
}

impl SetDetail {
    pub fn weight_display(&self) -> String {
        match self.weight {
            Some(w) => format!("{:.1}kg", w),
            None => "BW".to_string(),
        }
    }
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

pub async fn list_history(
    State(database_pool): State<SqlitePool>,
    session: Session,
) -> impl IntoResponse {
    let current_user = match get_current_user(&session, &database_pool).await {
        Some(user) => user,
        None => {
            return Html(
                r#"<meta http-equiv="refresh" content="0; url=/users">"#.to_string(),
            )
        }
    };

    let workouts = sqlx::query_as!(
        CompletedWorkoutWithName,
        r#"SELECT
            cw.id,
            cw.user_id,
            cw.workout_id,
            w.name as workout_name,
            cw.started_at,
            cw.completed_at,
            cw.total_duration_minutes as "total_duration_minutes: i32",
            cw.total_sets as "total_sets: i32",
            cw.total_volume_kg as "total_volume_kg: f32",
            cw.notes,
            cw.created_at
        FROM completed_workouts cw
        JOIN workouts w ON cw.workout_id = w.id
        WHERE cw.user_id = ?
        ORDER BY cw.completed_at DESC
        LIMIT 50"#,
        current_user.id
    )
    .fetch_all(&database_pool)
    .await
    .unwrap_or(Vec::new());

    let template = HistoryListTemplate {
        user: current_user.clone(),
        current_user: Some(current_user),
        workouts,
        is_dashboard: false,
    };

    Html(template.render().unwrap())
}

pub async fn show_history_detail(
    Path(workout_id): Path<String>,
    State(database_pool): State<SqlitePool>,
    session: Session,
) -> impl IntoResponse {
    let current_user = match get_current_user(&session, &database_pool).await {
        Some(user) => user,
        None => {
            return Html(
                r#"<meta http-equiv="refresh" content="0; url=/users">"#.to_string(),
            )
        }
    };

    // Get completed workout with name
    let workout = match sqlx::query_as!(
        CompletedWorkoutWithName,
        r#"SELECT
            cw.id,
            cw.user_id,
            cw.workout_id,
            w.name as workout_name,
            cw.started_at,
            cw.completed_at,
            cw.total_duration_minutes as "total_duration_minutes: i32",
            cw.total_sets as "total_sets: i32",
            cw.total_volume_kg as "total_volume_kg: f32",
            cw.notes,
            cw.created_at
        FROM completed_workouts cw
        JOIN workouts w ON cw.workout_id = w.id
        WHERE cw.id = ? AND cw.user_id = ?"#,
        workout_id,
        current_user.id
    )
    .fetch_optional(&database_pool)
    .await
    {
        Ok(Some(w)) => w,
        _ => {
            return Html(
                r#"<meta http-equiv="refresh" content="0; url=/history">"#.to_string(),
            )
        }
    };

    // Get all sets for this workout grouped by exercise
    let sets = sqlx::query!(
        r#"SELECT
            e.name as exercise_name,
            cs.set_number as "set_number: i32",
            cs.weight as "weight: f32",
            cs.reps as "reps: i32",
            cs.notes
        FROM completed_sets cs
        JOIN exercises e ON cs.exercise_id = e.id
        WHERE cs.active_workout_id = ?
        ORDER BY cs.completed_at"#,
        workout.id
    )
    .fetch_all(&database_pool)
    .await
    .unwrap_or(Vec::new());

    // Group sets by exercise
    let mut exercises: Vec<ExerciseWithSets> = Vec::new();
    for set in sets {
        let set_detail = SetDetail {
            set_number: set.set_number,
            weight: set.weight,
            reps: set.reps,
            notes: set.notes,
        };

        if let Some(exercise) = exercises
            .iter_mut()
            .find(|e| e.exercise_name == set.exercise_name)
        {
            exercise.sets.push(set_detail);
        } else {
            exercises.push(ExerciseWithSets {
                exercise_name: set.exercise_name,
                sets: vec![set_detail],
            });
        }
    }

    let template = HistoryDetailTemplate {
        user: current_user.clone(),
        current_user: Some(current_user),
        workout,
        exercises,
        is_dashboard: false,
    };

    Html(template.render().unwrap())
}

pub fn router() -> Router<SqlitePool> {
    Router::new()
        .route("/history", get(list_history))
        .route("/history/{id}", get(show_history_detail))
}
