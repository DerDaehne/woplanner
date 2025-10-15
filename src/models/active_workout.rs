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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn weight_display(&self) -> String {
        match self.weight {
            Some(w) => format!("{:.1}kg", w),
            None => "Bodyweight".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn set_display(&self) -> String {
        format!("{} x {}", self.weight_display(), self.reps)
    }

    #[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;

    // CompletedSet Tests
    #[test]
    fn test_completed_set_weight_display_with_weight() {
        let set = CompletedSet {
            id: "test-id".to_string(),
            active_workout_id: "workout-id".to_string(),
            exercise_id: "exercise-id".to_string(),
            set_number: 1,
            weight: Some(100.5),
            reps: 10,
            notes: None,
            completed_at: "2025-01-01T12:00:00Z".to_string(),
            created_at: "2025-01-01T12:00:00Z".to_string(),
        };
        assert_eq!(set.weight_display(), "100.5kg");
    }

    #[test]
    fn test_completed_set_weight_display_bodyweight() {
        let set = CompletedSet {
            id: "test-id".to_string(),
            active_workout_id: "workout-id".to_string(),
            exercise_id: "exercise-id".to_string(),
            set_number: 1,
            weight: None,
            reps: 15,
            notes: None,
            completed_at: "2025-01-01T12:00:00Z".to_string(),
            created_at: "2025-01-01T12:00:00Z".to_string(),
        };
        assert_eq!(set.weight_display(), "Bodyweight");
    }

    #[test]
    fn test_completed_set_weight_display_zero() {
        let set = CompletedSet {
            id: "test-id".to_string(),
            active_workout_id: "workout-id".to_string(),
            exercise_id: "exercise-id".to_string(),
            set_number: 1,
            weight: Some(0.0),
            reps: 10,
            notes: None,
            completed_at: "2025-01-01T12:00:00Z".to_string(),
            created_at: "2025-01-01T12:00:00Z".to_string(),
        };
        assert_eq!(set.weight_display(), "0.0kg");
    }

    #[test]
    fn test_completed_set_display() {
        let set = CompletedSet {
            id: "test-id".to_string(),
            active_workout_id: "workout-id".to_string(),
            exercise_id: "exercise-id".to_string(),
            set_number: 1,
            weight: Some(80.0),
            reps: 12,
            notes: None,
            completed_at: "2025-01-01T12:00:00Z".to_string(),
            created_at: "2025-01-01T12:00:00Z".to_string(),
        };
        assert_eq!(set.set_display(), "80.0kg x 12");
    }

    #[test]
    fn test_completed_set_display_bodyweight() {
        let set = CompletedSet {
            id: "test-id".to_string(),
            active_workout_id: "workout-id".to_string(),
            exercise_id: "exercise-id".to_string(),
            set_number: 1,
            weight: None,
            reps: 20,
            notes: None,
            completed_at: "2025-01-01T12:00:00Z".to_string(),
            created_at: "2025-01-01T12:00:00Z".to_string(),
        };
        assert_eq!(set.set_display(), "Bodyweight x 20");
    }

    #[test]
    fn test_completed_set_volume_with_weight() {
        let set = CompletedSet {
            id: "test-id".to_string(),
            active_workout_id: "workout-id".to_string(),
            exercise_id: "exercise-id".to_string(),
            set_number: 1,
            weight: Some(50.0),
            reps: 10,
            notes: None,
            completed_at: "2025-01-01T12:00:00Z".to_string(),
            created_at: "2025-01-01T12:00:00Z".to_string(),
        };
        assert_eq!(set.volume(), 500.0);
    }

    #[test]
    fn test_completed_set_volume_bodyweight() {
        let set = CompletedSet {
            id: "test-id".to_string(),
            active_workout_id: "workout-id".to_string(),
            exercise_id: "exercise-id".to_string(),
            set_number: 1,
            weight: None,
            reps: 10,
            notes: None,
            completed_at: "2025-01-01T12:00:00Z".to_string(),
            created_at: "2025-01-01T12:00:00Z".to_string(),
        };
        assert_eq!(set.volume(), 0.0);
    }

    #[test]
    fn test_completed_set_volume_zero_reps() {
        let set = CompletedSet {
            id: "test-id".to_string(),
            active_workout_id: "workout-id".to_string(),
            exercise_id: "exercise-id".to_string(),
            set_number: 1,
            weight: Some(100.0),
            reps: 0,
            notes: None,
            completed_at: "2025-01-01T12:00:00Z".to_string(),
            created_at: "2025-01-01T12:00:00Z".to_string(),
        };
        assert_eq!(set.volume(), 0.0);
    }

    // CompletedWorkout Tests
    #[test]
    fn test_completed_workout_duration_display_minutes() {
        let workout = CompletedWorkout {
            id: "test-id".to_string(),
            user_id: "user-id".to_string(),
            workout_id: "workout-id".to_string(),
            started_at: "2025-01-01T12:00:00Z".to_string(),
            completed_at: "2025-01-01T12:45:00Z".to_string(),
            total_duration_minutes: 45,
            total_sets: 10,
            total_volume_kg: 1000.0,
            notes: None,
            created_at: "2025-01-01T12:45:00Z".to_string(),
        };
        assert_eq!(workout.duration_display(), "45m");
    }

    #[test]
    fn test_completed_workout_duration_display_hours() {
        let workout = CompletedWorkout {
            id: "test-id".to_string(),
            user_id: "user-id".to_string(),
            workout_id: "workout-id".to_string(),
            started_at: "2025-01-01T12:00:00Z".to_string(),
            completed_at: "2025-01-01T14:30:00Z".to_string(),
            total_duration_minutes: 150,
            total_sets: 10,
            total_volume_kg: 1000.0,
            notes: None,
            created_at: "2025-01-01T14:30:00Z".to_string(),
        };
        assert_eq!(workout.duration_display(), "2h 30m");
    }

    #[test]
    fn test_completed_workout_duration_display_exact_hour() {
        let workout = CompletedWorkout {
            id: "test-id".to_string(),
            user_id: "user-id".to_string(),
            workout_id: "workout-id".to_string(),
            started_at: "2025-01-01T12:00:00Z".to_string(),
            completed_at: "2025-01-01T13:00:00Z".to_string(),
            total_duration_minutes: 60,
            total_sets: 10,
            total_volume_kg: 1000.0,
            notes: None,
            created_at: "2025-01-01T13:00:00Z".to_string(),
        };
        assert_eq!(workout.duration_display(), "1h 0m");
    }

    #[test]
    fn test_completed_workout_average_volume_per_set() {
        let workout = CompletedWorkout {
            id: "test-id".to_string(),
            user_id: "user-id".to_string(),
            workout_id: "workout-id".to_string(),
            started_at: "2025-01-01T12:00:00Z".to_string(),
            completed_at: "2025-01-01T13:00:00Z".to_string(),
            total_duration_minutes: 60,
            total_sets: 10,
            total_volume_kg: 1000.0,
            notes: None,
            created_at: "2025-01-01T13:00:00Z".to_string(),
        };
        assert_eq!(workout.average_volume_per_set(), 100.0);
    }

    #[test]
    fn test_completed_workout_average_volume_per_set_zero_sets() {
        let workout = CompletedWorkout {
            id: "test-id".to_string(),
            user_id: "user-id".to_string(),
            workout_id: "workout-id".to_string(),
            started_at: "2025-01-01T12:00:00Z".to_string(),
            completed_at: "2025-01-01T13:00:00Z".to_string(),
            total_duration_minutes: 60,
            total_sets: 0,
            total_volume_kg: 0.0,
            notes: None,
            created_at: "2025-01-01T13:00:00Z".to_string(),
        };
        assert_eq!(workout.average_volume_per_set(), 0.0);
    }

    #[test]
    fn test_completed_workout_average_volume_per_set_fractional() {
        let workout = CompletedWorkout {
            id: "test-id".to_string(),
            user_id: "user-id".to_string(),
            workout_id: "workout-id".to_string(),
            started_at: "2025-01-01T12:00:00Z".to_string(),
            completed_at: "2025-01-01T13:00:00Z".to_string(),
            total_duration_minutes: 60,
            total_sets: 3,
            total_volume_kg: 100.0,
            notes: None,
            created_at: "2025-01-01T13:00:00Z".to_string(),
        };
        assert!((workout.average_volume_per_set() - 33.333_332).abs() < 0.01);
    }

    // ActiveWorkout Tests
    #[test]
    fn test_active_workout_duration_display_minutes() {
        // Note: This test uses a fixed timestamp, so we need to create
        // a recent timestamp for realistic testing
        let now = chrono::Utc::now();
        let started_45_min_ago = now - chrono::Duration::minutes(45);

        let active_workout = ActiveWorkout {
            id: "test-id".to_string(),
            user_id: "user-id".to_string(),
            workout_id: "workout-id".to_string(),
            started_at: started_45_min_ago.to_rfc3339(),
            created_at: started_45_min_ago.to_rfc3339(),
        };

        let display = active_workout.duration_display();
        // Should be around 45 minutes (allowing for test execution time)
        assert!(display.contains("45m") || display.contains("44m") || display.contains("46m"));
    }

    #[test]
    fn test_active_workout_duration_display_hours() {
        let now = chrono::Utc::now();
        let started_2h_30m_ago = now - chrono::Duration::minutes(150);

        let active_workout = ActiveWorkout {
            id: "test-id".to_string(),
            user_id: "user-id".to_string(),
            workout_id: "workout-id".to_string(),
            started_at: started_2h_30m_ago.to_rfc3339(),
            created_at: started_2h_30m_ago.to_rfc3339(),
        };

        let display = active_workout.duration_display();
        assert!(display.contains("2h") && display.contains("30m") || display.contains("2h 29m"));
    }

    #[test]
    fn test_active_workout_duration_display_invalid_timestamp() {
        let active_workout = ActiveWorkout {
            id: "test-id".to_string(),
            user_id: "user-id".to_string(),
            workout_id: "workout-id".to_string(),
            started_at: "invalid-timestamp".to_string(),
            created_at: "invalid-timestamp".to_string(),
        };

        assert_eq!(active_workout.duration_display(), "0m");
    }

    #[test]
    fn test_active_workout_duration_minutes_valid() {
        let now = chrono::Utc::now();
        let started_30_min_ago = now - chrono::Duration::minutes(30);

        let active_workout = ActiveWorkout {
            id: "test-id".to_string(),
            user_id: "user-id".to_string(),
            workout_id: "workout-id".to_string(),
            started_at: started_30_min_ago.to_rfc3339(),
            created_at: started_30_min_ago.to_rfc3339(),
        };

        let duration = active_workout.duration_minutes();
        assert!(duration.is_some());
        let mins = duration.unwrap();
        assert!(mins >= 29 && mins <= 31); // Allow for test execution time
    }

    #[test]
    fn test_active_workout_duration_minutes_invalid() {
        let active_workout = ActiveWorkout {
            id: "test-id".to_string(),
            user_id: "user-id".to_string(),
            workout_id: "workout-id".to_string(),
            started_at: "invalid-timestamp".to_string(),
            created_at: "invalid-timestamp".to_string(),
        };

        assert!(active_workout.duration_minutes().is_none());
    }
}
