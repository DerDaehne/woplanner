use ::chrono::Datelike;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, types::chrono};
use uuid::Uuid;

// different schedule types for workout planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleType {
    Manual,   // user chooses workout manually
    Weekly,   // workout gets scheduled on a weekly basis
    Rotation, // workouts are in a global rotation
    Disabled, // workout is disabled
}

impl ScheduleType {
    #[allow(dead_code)]
    pub fn to_string(&self) -> String {
        match self {
            ScheduleType::Manual => "manual".to_string(),
            ScheduleType::Weekly => "weekly".to_string(),
            ScheduleType::Rotation => "rotation".to_string(),
            ScheduleType::Disabled => "disabled".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "weekly" => ScheduleType::Weekly,
            "rotation" => ScheduleType::Rotation,
            "disabled" => ScheduleType::Disabled,
            _ => ScheduleType::Manual,
        }
    }
}

// a single workout
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Workout {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub schedule_type: String,
    pub schedule_day: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

// a single exercise as part of an workout
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkoutExercise {
    pub id: String,
    pub workout_id: String,
    pub exercise_id: String,
    pub position: i32,
    pub target_sets: i32,
    pub target_weight: Option<f32>,
    pub notes: Option<String>,
    pub created_at: String,
}

// view model: workout with exercises
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkoutWithExercises {
    pub workout: Workout,
    pub exercises: Vec<WorkoutExercise>,
}

impl Workout {
    pub fn new(user_id: String, name: String, description: Option<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            user_id,
            name,
            description,
            is_active: Some(true),
            schedule_type: "manual".to_string(),
            schedule_day: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    #[allow(dead_code)]
    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

    #[allow(dead_code)]
    pub fn get_schedule_type(&self) -> ScheduleType {
        ScheduleType::from_str(&self.schedule_type)
    }

    pub fn day_name(&self) -> Option<String> {
        self.schedule_day.map(|day| {
            match day {
                0 => "Sonntag",
                1 => "Montag",
                2 => "Dienstag",
                3 => "Mittwoch",
                4 => "Donnerstag",
                5 => "Freitag",
                6 => "Samstag",
                _ => "Unknown",
            }
            .to_string()
        })
    }

    pub fn is_scheduled_today(&self) -> bool {
        if self.schedule_type == "weekly"
            && let Some(day) = self.schedule_day
        {
            let today = chrono::Utc::now().weekday().num_days_from_sunday() as i32;
            return day == today;
        }
        false
    }
}

impl WorkoutExercise {
    pub fn new(
        workout_id: String,
        exercise_id: String,
        position: i32,
        target_sets: i32,
        target_weight: Option<f32>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            workout_id,
            exercise_id,
            position,
            target_sets,
            target_weight,
            notes: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }

    #[allow(dead_code)]
    pub fn weight_display(&self) -> String {
        match self.target_weight {
            Some(weight) => format!("{:.1} kg", weight),
            None => "Bodyweight".to_string(),
        }
    }
}

// form models
#[derive(Debug, Deserialize)]
pub struct CreateWorkoutForm {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddExerciseToWorkoutForm {
    pub exercise_id: String,
    pub target_sets: i32,
    pub target_weight: Option<f32>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkoutScheduleForm {
    pub schedule_type: String,
    pub schedule_day: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ScheduleType Tests
    #[test]
    fn test_schedule_type_to_string() {
        assert_eq!(ScheduleType::Manual.to_string(), "manual");
        assert_eq!(ScheduleType::Weekly.to_string(), "weekly");
        assert_eq!(ScheduleType::Rotation.to_string(), "rotation");
        assert_eq!(ScheduleType::Disabled.to_string(), "disabled");
    }

    #[test]
    fn test_schedule_type_from_str() {
        matches!(ScheduleType::from_str("manual"), ScheduleType::Manual);
        matches!(ScheduleType::from_str("weekly"), ScheduleType::Weekly);
        matches!(ScheduleType::from_str("rotation"), ScheduleType::Rotation);
        matches!(ScheduleType::from_str("disabled"), ScheduleType::Disabled);
    }

    #[test]
    fn test_schedule_type_from_str_invalid() {
        // Invalid strings should default to Manual
        matches!(ScheduleType::from_str("invalid"), ScheduleType::Manual);
        matches!(ScheduleType::from_str(""), ScheduleType::Manual);
        matches!(ScheduleType::from_str("WEEKLY"), ScheduleType::Manual);
    }

    // Workout Tests
    #[test]
    fn test_workout_day_name_all_days() {
        let mut workout = Workout::new(
            "user-id".to_string(),
            "Test Workout".to_string(),
            None,
        );

        workout.schedule_day = Some(0);
        assert_eq!(workout.day_name(), Some("Sonntag".to_string()));

        workout.schedule_day = Some(1);
        assert_eq!(workout.day_name(), Some("Montag".to_string()));

        workout.schedule_day = Some(2);
        assert_eq!(workout.day_name(), Some("Dienstag".to_string()));

        workout.schedule_day = Some(3);
        assert_eq!(workout.day_name(), Some("Mittwoch".to_string()));

        workout.schedule_day = Some(4);
        assert_eq!(workout.day_name(), Some("Donnerstag".to_string()));

        workout.schedule_day = Some(5);
        assert_eq!(workout.day_name(), Some("Freitag".to_string()));

        workout.schedule_day = Some(6);
        assert_eq!(workout.day_name(), Some("Samstag".to_string()));
    }

    #[test]
    fn test_workout_day_name_invalid() {
        let mut workout = Workout::new(
            "user-id".to_string(),
            "Test Workout".to_string(),
            None,
        );

        workout.schedule_day = Some(7);
        assert_eq!(workout.day_name(), Some("Unknown".to_string()));

        workout.schedule_day = Some(-1);
        assert_eq!(workout.day_name(), Some("Unknown".to_string()));

        workout.schedule_day = Some(100);
        assert_eq!(workout.day_name(), Some("Unknown".to_string()));
    }

    #[test]
    fn test_workout_day_name_none() {
        let workout = Workout::new(
            "user-id".to_string(),
            "Test Workout".to_string(),
            None,
        );

        assert_eq!(workout.day_name(), None);
    }

    #[test]
    fn test_workout_is_scheduled_today() {
        let today = chrono::Utc::now().weekday().num_days_from_sunday() as i32;

        let mut workout = Workout::new(
            "user-id".to_string(),
            "Test Workout".to_string(),
            None,
        );
        workout.schedule_type = "weekly".to_string();
        workout.schedule_day = Some(today);

        assert!(workout.is_scheduled_today());
    }

    #[test]
    fn test_workout_is_not_scheduled_today_wrong_day() {
        let today = chrono::Utc::now().weekday().num_days_from_sunday() as i32;
        let different_day = (today + 1) % 7; // Next day

        let mut workout = Workout::new(
            "user-id".to_string(),
            "Test Workout".to_string(),
            None,
        );
        workout.schedule_type = "weekly".to_string();
        workout.schedule_day = Some(different_day);

        assert!(!workout.is_scheduled_today());
    }

    #[test]
    fn test_workout_is_not_scheduled_today_wrong_type() {
        let today = chrono::Utc::now().weekday().num_days_from_sunday() as i32;

        let mut workout = Workout::new(
            "user-id".to_string(),
            "Test Workout".to_string(),
            None,
        );
        workout.schedule_type = "manual".to_string();
        workout.schedule_day = Some(today);

        assert!(!workout.is_scheduled_today());
    }

    #[test]
    fn test_workout_is_not_scheduled_today_no_day() {
        let mut workout = Workout::new(
            "user-id".to_string(),
            "Test Workout".to_string(),
            None,
        );
        workout.schedule_type = "weekly".to_string();
        workout.schedule_day = None;

        assert!(!workout.is_scheduled_today());
    }

    #[test]
    fn test_workout_get_schedule_type() {
        let mut workout = Workout::new(
            "user-id".to_string(),
            "Test Workout".to_string(),
            None,
        );

        workout.schedule_type = "manual".to_string();
        matches!(workout.get_schedule_type(), ScheduleType::Manual);

        workout.schedule_type = "weekly".to_string();
        matches!(workout.get_schedule_type(), ScheduleType::Weekly);

        workout.schedule_type = "rotation".to_string();
        matches!(workout.get_schedule_type(), ScheduleType::Rotation);

        workout.schedule_type = "disabled".to_string();
        matches!(workout.get_schedule_type(), ScheduleType::Disabled);
    }

    #[test]
    fn test_workout_new_defaults() {
        let workout = Workout::new(
            "user-123".to_string(),
            "My Workout".to_string(),
            Some("Description".to_string()),
        );

        assert_eq!(workout.user_id, "user-123");
        assert_eq!(workout.name, "My Workout");
        assert_eq!(workout.description, Some("Description".to_string()));
        assert_eq!(workout.is_active, Some(true));
        assert_eq!(workout.schedule_type, "manual");
        assert_eq!(workout.schedule_day, None);
        assert!(!workout.id.is_empty());
        assert!(!workout.created_at.is_empty());
        assert!(!workout.updated_at.is_empty());
    }

    #[test]
    fn test_workout_touch_updates_timestamp() {
        let mut workout = Workout::new(
            "user-id".to_string(),
            "Test Workout".to_string(),
            None,
        );

        let original_updated_at = workout.updated_at.clone();

        // Sleep briefly to ensure timestamp difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        workout.touch();

        assert_ne!(workout.updated_at, original_updated_at);
    }

    // WorkoutExercise Tests
    #[test]
    fn test_workout_exercise_weight_display_with_weight() {
        let exercise = WorkoutExercise::new(
            "workout-id".to_string(),
            "exercise-id".to_string(),
            1,
            3,
            Some(100.0),
        );

        assert_eq!(exercise.weight_display(), "100.0 kg");
    }

    #[test]
    fn test_workout_exercise_weight_display_bodyweight() {
        let exercise = WorkoutExercise::new(
            "workout-id".to_string(),
            "exercise-id".to_string(),
            1,
            3,
            None,
        );

        assert_eq!(exercise.weight_display(), "Bodyweight");
    }

    #[test]
    fn test_workout_exercise_new_defaults() {
        let exercise = WorkoutExercise::new(
            "workout-123".to_string(),
            "exercise-456".to_string(),
            2,
            5,
            Some(80.5),
        );

        assert_eq!(exercise.workout_id, "workout-123");
        assert_eq!(exercise.exercise_id, "exercise-456");
        assert_eq!(exercise.position, 2);
        assert_eq!(exercise.target_sets, 5);
        assert_eq!(exercise.target_weight, Some(80.5));
        assert_eq!(exercise.notes, None);
        assert!(!exercise.id.is_empty());
        assert!(!exercise.created_at.is_empty());
    }
}
