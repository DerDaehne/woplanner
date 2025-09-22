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
    pub fn to_string(&self) -> String {
        match self {
            ScheduleType::Manual => "manual".to_string(),
            ScheduleType::Weekly => "weekly".to_string(),
            ScheduleType::Rotation => "rotation".to_string(),
            ScheduleType::Disabled => "disabled".to_string(),
        }
    }

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

    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }

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

    pub fn weight_display(&self) -> String {
        match self.target_weight {
            Some(weight) => format!("{:.1}; kg", weight),
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
