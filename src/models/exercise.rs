use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Exercise {
    pub id: String,
    pub name: String,
    pub instructions: String,
    pub video_url: Option<String>,
    pub created_at: String,
}

impl Exercise {
    pub fn new(name: String, instructions: String, video_url: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            instructions,
            video_url,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}
