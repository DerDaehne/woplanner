use crate::models::CompletedSet;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

/// Check and update personal records after a set is completed
pub async fn check_and_update_prs(
    pool: &SqlitePool,
    user_id: &str,
    exercise_id: &str,
    completed_set: &CompletedSet,
) -> Result<Vec<String>, sqlx::Error> {
    let mut achieved_prs = Vec::new();

    // Check max weight PR (only for weighted exercises)
    if let Some(weight) = completed_set.weight {
        let current_max = sqlx::query_scalar::<_, Option<f64>>(
            "SELECT weight FROM personal_records
             WHERE user_id = ? AND exercise_id = ? AND record_type = 'max_weight'"
        )
        .bind(user_id)
        .bind(exercise_id)
        .fetch_optional(pool)
        .await?
        .flatten();

        let weight_f64 = weight as f64;

        if current_max.is_none() || weight_f64 > current_max.unwrap() {
            // New PR for max weight!
            let pr_id = Uuid::new_v4().to_string();
            let now = Utc::now().to_rfc3339();

            sqlx::query(
                "INSERT OR REPLACE INTO personal_records
                 (id, user_id, exercise_id, record_type, weight, reps, volume_kg, completed_set_id, achieved_at, created_at)
                 VALUES (?, ?, ?, 'max_weight', ?, ?, ?, ?, ?, ?)"
            )
            .bind(&pr_id)
            .bind(user_id)
            .bind(exercise_id)
            .bind(weight_f64)
            .bind(completed_set.reps as i64)
            .bind((weight_f64 * completed_set.reps as f64))
            .bind(&completed_set.id)
            .bind(&now)
            .bind(&now)
            .execute(pool)
            .await?;

            achieved_prs.push(format!("Max Weight: {}kg", weight));
        }
    }

    // Check max reps PR
    let current_max_reps = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT reps FROM personal_records
         WHERE user_id = ? AND exercise_id = ? AND record_type = 'max_reps'"
    )
    .bind(user_id)
    .bind(exercise_id)
    .fetch_optional(pool)
    .await?
    .flatten();

    if current_max_reps.is_none() || (completed_set.reps as i64) > current_max_reps.unwrap() {
        // New PR for max reps!
        let pr_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let weight_f64 = completed_set.weight.map(|w| w as f64);
        let volume = weight_f64.map(|w| w * completed_set.reps as f64);

        sqlx::query(
            "INSERT OR REPLACE INTO personal_records
             (id, user_id, exercise_id, record_type, weight, reps, volume_kg, completed_set_id, achieved_at, created_at)
             VALUES (?, ?, ?, 'max_reps', ?, ?, ?, ?, ?, ?)"
        )
        .bind(&pr_id)
        .bind(user_id)
        .bind(exercise_id)
        .bind(weight_f64)
        .bind(completed_set.reps as i64)
        .bind(volume)
        .bind(&completed_set.id)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;

        achieved_prs.push(format!("Max Reps: {}", completed_set.reps));
    }

    // Check max volume PR (only for weighted exercises)
    if let Some(weight) = completed_set.weight {
        let volume = (weight as f64) * (completed_set.reps as f64);

        let current_max_volume = sqlx::query_scalar::<_, Option<f64>>(
            "SELECT volume_kg FROM personal_records
             WHERE user_id = ? AND exercise_id = ? AND record_type = 'max_volume'"
        )
        .bind(user_id)
        .bind(exercise_id)
        .fetch_optional(pool)
        .await?
        .flatten();

        if current_max_volume.is_none() || volume > current_max_volume.unwrap() {
            // New PR for max volume!
            let pr_id = Uuid::new_v4().to_string();
            let now = Utc::now().to_rfc3339();

            sqlx::query(
                "INSERT OR REPLACE INTO personal_records
                 (id, user_id, exercise_id, record_type, weight, reps, volume_kg, completed_set_id, achieved_at, created_at)
                 VALUES (?, ?, ?, 'max_volume', ?, ?, ?, ?, ?, ?)"
            )
            .bind(&pr_id)
            .bind(user_id)
            .bind(exercise_id)
            .bind(weight as f64)
            .bind(completed_set.reps as i64)
            .bind(volume)
            .bind(&completed_set.id)
            .bind(&now)
            .bind(&now)
            .execute(pool)
            .await?;

            achieved_prs.push(format!("Max Volume: {:.1}kg", volume));
        }
    }

    Ok(achieved_prs)
}
