use std::path::Path;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use poise::serenity_prelude::UserId;
use tokio::fs;

const BUTTON_STATS_FILE: &str = "button_stats.json";

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ButtonStats {
    // Map of UserId -> button press count
    pub press_counts: HashMap<UserId, u64>,
}

impl ButtonStats {
    /// Loads the button statistics from `button_stats.json`.
    /// If the file doesn't exist, creates a new empty stats file.
    pub async fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let path = Path::new(BUTTON_STATS_FILE);

        if !path.exists() {
            let default_stats = ButtonStats::default();
            default_stats.save().await?;
            return Ok(default_stats);
        }

        let content = fs::read_to_string(path).await?;
        let stats: ButtonStats = serde_json::from_str(&content)?;
        Ok(stats)
    }

    /// Saves the current button statistics to `button_stats.json`.
    pub async fn save(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(BUTTON_STATS_FILE, content).await?;
        Ok(())
    }

    /// Increments the press count for a given user.
    pub fn increment(&mut self, user_id: UserId) {
        let count = self.press_counts.entry(user_id).or_insert(0);
        *count += 1;
    }

    /// Gets the press count for a given user.
    pub fn get_count(&self, user_id: UserId) -> u64 {
        *self.press_counts.get(&user_id).unwrap_or(&0)
    }
}
