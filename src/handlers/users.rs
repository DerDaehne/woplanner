use crate::error::AppError;
use crate::models::{ActiveWorkout, CompletedWorkout, Exercise, User};
use askama::Template;
use axum::{
    Form, Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue},
    response::{Html, IntoResponse},
    routing::{get, post},
};
use serde::Deserialize;
use sqlx::SqlitePool;
use tower_sessions::Session;

#[derive(Debug, Deserialize)]
pub struct CreateUserForm {
    pub name: String,
}

#[derive(Template)]
#[template(path = "users/list.html")]
pub struct UserListTemplate {
    pub users: Vec<User>,
    pub current_user: Option<User>,
    pub is_dashboard: bool,
}

#[derive(Template)]
#[template(path = "users/user_list_partial.html")]
pub struct UserListPartialTemplate {
    pub users: Vec<User>,
}

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub user: User,
    pub current_user: Option<User>,
    pub exercises: Vec<Exercise>,
    pub recent_workouts: Vec<CompletedWorkout>,
    pub active_workout: Option<ActiveWorkout>,
    pub stats: DashboardStats,
    pub is_dashboard: bool,
}

#[derive(Debug, Clone)]
pub struct DashboardStats {
    pub current_streak: i32,
    pub workouts_this_week: i32,
    pub total_workouts: i32,
    pub last_workout: Option<String>,
    pub total_volume_kg: f32,
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

pub async fn list_users(
    State(database_pool): State<SqlitePool>,
    session: Session,
) -> Result<impl IntoResponse, AppError> {
    let users = sqlx::query_as!(User, "select * from users;")
        .fetch_all(&database_pool)
        .await?;

    let current_user = get_current_user(&session, &database_pool).await?;

    let template = UserListTemplate {
        users,
        current_user,
        is_dashboard: false,
    };
    Ok(Html(template.render()?).into_response())
}

pub async fn create_user(
    State(database_pool): State<SqlitePool>,
    Form(form_data): Form<CreateUserForm>,
) -> Result<impl IntoResponse, AppError> {
    // Validate input
    let name = form_data.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::BadRequest("Name cannot be empty".to_string()));
    }

    let new_user = User::new(name.clone());
    sqlx::query_as!(
        User,
        "INSERT INTO users (id, name, created_at) VALUES (?, ?, ?)",
        new_user.id,
        new_user.name,
        new_user.created_at
    )
    .execute(&database_pool)
    .await?;

    let users = sqlx::query_as!(User, "SELECT * from users")
        .fetch_all(&database_pool)
        .await?;
    let template = UserListPartialTemplate {
        users: users.clone(),
    };
    Ok(Html(template.render()?).into_response())
}

pub async fn select_user(
    Path(user_id): Path<String>,
    State(database_pool): State<SqlitePool>,
    session: Session,
) -> Result<impl IntoResponse, AppError> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", user_id)
        .fetch_optional(&database_pool)
        .await?;

    match user {
        Some(user) => {
            session
                .insert("current_user_id", &user.id)
                .await?;
            tracing::info!("user selected: {}", user.name);

            let mut headers = HeaderMap::new();
            headers.insert("HX-Redirect", HeaderValue::from_static("/dashboard"));

            Ok((
                headers,
                Html(format!(
                    r#"<div class="bg-green-500 text-white px-4 py-2 rounded-md shadow-md">
                    ✅ {} ausgewählt! Weiterleitung...
                </div>"#,
                    user.name
                )),
            ).into_response())
        }
        None => {
            let headers = HeaderMap::new();
            Ok((
                headers,
                Html(
                    r#"<div class="bg-red-500 text-white px-4 py-2 rounded-md shadow-md">
                    ❌ User nicht gefunden!
                </div>"#
                            .to_string(),
                ),
            ).into_response())
        }
    }
}

pub async fn dashboard(
    State(database_pool): State<SqlitePool>,
    session: Session,
) -> Result<impl IntoResponse, AppError> {
    let user_id = match session.get::<String>("current_user_id").await? {
        Some(id) => id,
        None => return Ok(Html(r#"<p>You are not logged in. <a href="/users">Log in now</a></p>"#.to_string()).into_response()),
    };

    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", user_id)
        .fetch_optional(&database_pool)
        .await?;

    let user = match user {
        Some(u) => u,
        None => return Ok(Html(r#"<p>Session invalid. <a href="/users">Please log in again</a></p>"#.to_string()).into_response()),
    };

    let recent_workouts = sqlx::query_as!(
        CompletedWorkout,
        r#"SELECT
            id,
            user_id,
            workout_id,
            started_at,
            completed_at,
            total_duration_minutes as "total_duration_minutes: i32",
            total_sets as "total_sets: i32",
            total_volume_kg as "total_volume_kg: f32",
            notes,
            created_at
            FROM completed_workouts
            WHERE user_id = ?
            ORDER BY completed_at DESC LIMIT 3"#,
        user.id
    )
    .fetch_all(&database_pool)
    .await?;

    let active_workout = sqlx::query_as!(
        ActiveWorkout,
        "SELECT * FROM active_workouts WHERE user_id = ? LIMIT 1",
        user.id
    )
    .fetch_optional(&database_pool)
    .await?;

    let total_workouts = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM completed_workouts WHERE user_id = ?",
        user.id
    )
    .fetch_one(&database_pool)
    .await
    .unwrap_or(0) as i32;

    let total_volume_kg = sqlx::query_scalar!(
        r#"SELECT COALESCE(SUM(total_volume_kg), 0.0) as "total!: f32" 
           FROM completed_workouts WHERE user_id = ?"#,
        user.id
    )
    .fetch_one(&database_pool)
    .await
    .unwrap_or(0.0);

    let workouts_this_week = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM completed_workouts 
         WHERE user_id = ? 
         AND datetime(completed_at) >= datetime('now', '-7 days')",
        user.id
    )
    .fetch_one(&database_pool)
    .await
    .unwrap_or(0) as i32;

    let current_streak = if workouts_this_week > 0 {
        workouts_this_week
    } else {
        0
    };

    let last_workout = recent_workouts.first().map(|w| {
        let completed = chrono::DateTime::parse_from_rfc3339(&w.completed_at)
            .unwrap_or_else(|_| chrono::Utc::now().into());
        let now = chrono::Utc::now();
        let diff = now.signed_duration_since(completed).num_days();

        match diff {
            0 => "Today".to_string(),
            1 => "Yesterday".to_string(),
            n => format!("{} days ago", n),
        }
    });

    let stats = DashboardStats {
        current_streak,
        workouts_this_week,
        total_workouts,
        last_workout,
        total_volume_kg,
    };

    let exercises =
        sqlx::query_as!(Exercise, "SELECT id, name, instructions, video_url, created_at FROM exercises ORDER BY name LIMIT 3")
            .fetch_all(&database_pool)
            .await?;

    let template = DashboardTemplate {
        user: user.clone(),
        current_user: Some(user),
        exercises,
        recent_workouts,
        active_workout,
        stats,
        is_dashboard: true,
    };
    Ok(Html(template.render()?).into_response())
}

pub async fn logout(session: Session) -> Result<impl IntoResponse, AppError> {
    session.flush().await?;
    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", HeaderValue::from_static("/users"));
    Ok((
        headers,
        Html(r#"<meta http-equiv="refresh" content="0; url=/users">"#.to_string()),
    ).into_response())
}

pub fn router() -> Router<SqlitePool> {
    Router::new()
        .route("/users", get(list_users))
        .route("/users", post(create_user))
        .route("/users/{id}/select", post(select_user))
        .route("/dashboard", get(dashboard))
        .route("/logout", post(logout))
}
