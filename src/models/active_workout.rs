use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ActiveWorkout {
    pub id: String,
    pub user_id: String,
    pub workout_id: String,
    pub started_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CompletedSet {
    pub id: String,
    pub active_workout_id: String,
    pub exercise_id: String,
    pub set_number: i32,
    pub weight: Option<f32>,
    pub reps: i32,
    pub notes: Option<String>,
    pub completed_at: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct CompletedWorkout {
    pub id: String,
    pub user_id: String,
    pub workout_id: String,
    pub started_at: String,
    pub completed_at: String,
    pub total_duration_minutes: i32,
    pub total_sets: i32,
    pub total_volume_kg: f32,
    pub notes: Option<String>,
    pub created_at: String,
}

// view modeL: exercise info
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkoutExerciseDetail {
    pub position: i32,
    pub target_sets: i32,
    pub target_weight: Option<f32>,
    pub notes: Option<String>,
    pub exercise_id: String,
    pub exercise_name: String,
    pub exercise_instructions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveWorkoutView {
    pub active_workout: ActiveWorkout,
    pub workout_name: String,
    pub total_sets_completed: i32,
    pub current_exercise: Option<WorkoutExerciseDetail>,
    pub progress_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedSetDetail {
    pub id: String,
    pub set_number: i32,
    pub weight: Option<f32>,
    pub reps: i32,
    pub notes: Option<String>,
    pub completed_at: String,
    pub exercise_name: String,
    pub exercise_instructions: String,
}

impl CompletedWorkout {
    pub fn new(
        active_workout: ActiveWorkout,
        total_sets: i32,
        total_volume_kg: f32,
        notes: Option<String>,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        let started = chrono::DateTime::parse_from_rfc3339(&active_workout.started_at)
            .unwrap_or_else(|_| chrono::Utc::now().into());
        let duration_minutes = chrono::Utc::now()
            .signed_duration_since(started)
            .num_minutes() as i32;

        Self {
            id: active_workout.id,
            user_id: active_workout.user_id,
            workout_id: active_workout.workout_id,
            started_at: active_workout.started_at,
            completed_at: now.clone(),
            total_duration_minutes: duration_minutes,
            total_sets,
            total_volume_kg,
            notes,
            created_at: now,
        }
    }

    pub fn duration_display(&self) -> String {
        let mins = self.total_duration_minutes;
        if mins < 60 {
            format!("{}m", mins)
        } else {
            format!("{}h {}m", mins / 60, mins % 60)
        }
    }

    pub fn average_volume_per_set(&self) -> f32 {
        if self.total_sets > 0 {
            self.total_volume_kg / self.total_sets as f32
        } else {
            0.0
        }
    }
}

impl ActiveWorkout {
    pub fn new(user_id: String, workout_id: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            workout_id,
            started_at: now.clone(),
            created_at: now,
        }
    }

    pub fn duration_minutes(&self) -> Option<i64> {
        let started = chrono::DateTime::parse_from_rfc3339(&self.started_at).ok()?;
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(started);
        Some(duration.num_minutes())
    }

    pub fn duration_display(&self) -> String {
        match self.duration_minutes() {
            Some(mins) if mins < 60 => format!("{}m", mins),
            Some(mins) => format!("{}h {}m", mins / 60, mins % 60),
            None => "0m".to_string(),
        }
    }
}

impl CompletedSet {
    pub fn new(
        active_workout_id: String,
        exercise_id: String,
        set_number: i32,
        weight: Option<f32>,
        reps: i32,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            active_workout_id,
            exercise_id,
            set_number,
            weight,
            reps,
            notes: None,
            completed_at: now.clone(),
            created_at: now,
        }
    }

    pub fn weight_display(&self) -> String {
        match self.weight {
            Some(w) => format!("{:.1}kg", w),
            None => "Bodyweight".to_string(),
        }
    }

    pub fn set_display(&self) -> String {
        format!("{} x {}", self.weight_display(), self.reps)
    }

    pub fn volume(&self) -> f32 {
        self.weight.unwrap_or(0.0) * self.reps as f32
    }
}

#[derive(Debug, Deserialize)]
pub struct StartWorkoutForm {
    pub workout_id: String,
}

#[derive(Debug, Deserialize)]
pub struct CompleteSetForm {
    pub exercise_id: String,
    pub weight: Option<f32>,
    pub reps: i32,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FinishTrainingForm {
    pub notes: Option<String>,
}
