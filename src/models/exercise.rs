use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Exercise {
    pub id: String,
    pub name: String,
    pub instructions: String,
    pub created_at: String,
}

impl Exercise {
    pub fn new(name: String, instructions: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            instructions,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}
