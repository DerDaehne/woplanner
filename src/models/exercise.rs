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

    /// Extracts YouTube video ID from URL for embedding
    /// Supports formats: youtube.com/watch?v=ID, youtu.be/ID
    pub fn youtube_embed_id(&self) -> Option<String> {
        let url = self.video_url.as_ref()?;

        // Handle youtube.com/watch?v=VIDEO_ID
        if let Some(pos) = url.find("v=") {
            let id_start = pos + 2;
            let id = url[id_start..]
                .split('&')
                .next()
                .unwrap_or("")
                .to_string();
            if !id.is_empty() {
                return Some(id);
            }
        }

        // Handle youtu.be/VIDEO_ID
        if url.contains("youtu.be/") {
            if let Some(pos) = url.rfind('/') {
                let id = url[pos + 1..]
                    .split('?')
                    .next()
                    .unwrap_or("")
                    .to_string();
                if !id.is_empty() {
                    return Some(id);
                }
            }
        }

        None
    }
}
