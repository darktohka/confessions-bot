use std::path::Path;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use poise::serenity_prelude::{ChannelId, GuildId, UserId};
use tokio::fs;

const CONFIG_FILE: &str = "config.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub discord_token: String,
    // Map of GuildId -> ChannelId (the thread where new confession threads are created)
    pub confession_threads: HashMap<GuildId, ChannelId>,
    // Map of GuildId -> cooldown in seconds (default: 3600 seconds = 1 hour)
    #[serde(default = "default_cooldowns")]
    pub cooldowns: HashMap<GuildId, u64>,
    // Map of (GuildId, UserId) -> timestamp of last confession
    #[serde(skip)]
    pub user_cooldowns: HashMap<(GuildId, UserId), i64>,
}

fn default_cooldowns() -> HashMap<GuildId, u64> {
    HashMap::new()
}

impl Config {
    /// Loads the configuration from `config.json`. If the file doesn't exist, 
    /// it creates a default one and prompts the user to fill it.
    pub async fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let path = Path::new(CONFIG_FILE);

        if !path.exists() {
            let default_config = Config {
                discord_token: "YOUR_BOT_TOKEN_HERE".to_string(),
                confession_threads: HashMap::new(),
                cooldowns: HashMap::new(),
                user_cooldowns: HashMap::new(),
            };
            default_config.save().await?;
            
            eprintln!("Created default {}. Please fill in your bot token.", CONFIG_FILE);
            return Err("Configuration file created. Please update it and restart.".into());
        }

        let content = fs::read_to_string(path).await?;
        let mut config: Config = serde_json::from_str(&content)?;
        
        // Initialize user_cooldowns as it's skipped during deserialization
        config.user_cooldowns = HashMap::new();
        
        if config.discord_token == "YOUR_BOT_TOKEN_HERE" {
            return Err("Please replace YOUR_BOT_TOKEN_HERE in config.json with your actual bot token.".into());
        }

        Ok(config)
    }

    /// Saves the current configuration state to `config.json`.
    pub async fn save(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(CONFIG_FILE, content).await?;
        Ok(())
    }
    
    /// Gets the cooldown period for a guild in seconds. Returns 3600 (1 hour) by default.
    pub fn get_cooldown(&self, guild_id: GuildId) -> u64 {
        self.cooldowns.get(&guild_id).copied().unwrap_or(3600)
    }
    
    /// Checks if a user is on cooldown. Returns None if not on cooldown (or cooldown is disabled),
    /// or Some(seconds_remaining) if still on cooldown.
    /// When cooldown is set to 0, it is effectively disabled and always returns None.
    pub fn check_cooldown(&self, guild_id: GuildId, user_id: UserId, current_time: i64) -> Option<i64> {
        let cooldown_seconds = self.get_cooldown(guild_id) as i64;
        
        // Cooldown of 0 means disabled
        if cooldown_seconds == 0 {
            return None;
        }
        
        if let Some(&last_submission) = self.user_cooldowns.get(&(guild_id, user_id)) {
            let time_elapsed = current_time - last_submission;
            if time_elapsed < cooldown_seconds {
                return Some(cooldown_seconds - time_elapsed);
            }
        }
        
        None
    }
    
    /// Records a confession submission time for a user.
    pub fn record_submission(&mut self, guild_id: GuildId, user_id: UserId, timestamp: i64) {
        self.user_cooldowns.insert((guild_id, user_id), timestamp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_cooldown() {
        let config = Config {
            discord_token: "test".to_string(),
            confession_threads: HashMap::new(),
            cooldowns: HashMap::new(),
            user_cooldowns: HashMap::new(),
        };
        
        let guild_id = GuildId::new(12345);
        assert_eq!(config.get_cooldown(guild_id), 3600);
    }

    #[test]
    fn test_custom_cooldown() {
        let mut config = Config {
            discord_token: "test".to_string(),
            confession_threads: HashMap::new(),
            cooldowns: HashMap::new(),
            user_cooldowns: HashMap::new(),
        };
        
        let guild_id = GuildId::new(12345);
        config.cooldowns.insert(guild_id, 7200);
        assert_eq!(config.get_cooldown(guild_id), 7200);
    }

    #[test]
    fn test_cooldown_check_no_prior_submission() {
        let config = Config {
            discord_token: "test".to_string(),
            confession_threads: HashMap::new(),
            cooldowns: HashMap::new(),
            user_cooldowns: HashMap::new(),
        };
        
        let guild_id = GuildId::new(12345);
        let user_id = UserId::new(67890);
        let current_time = 1000;
        
        assert_eq!(config.check_cooldown(guild_id, user_id, current_time), None);
    }

    #[test]
    fn test_cooldown_check_within_cooldown() {
        let mut config = Config {
            discord_token: "test".to_string(),
            confession_threads: HashMap::new(),
            cooldowns: HashMap::new(),
            user_cooldowns: HashMap::new(),
        };
        
        let guild_id = GuildId::new(12345);
        let user_id = UserId::new(67890);
        config.cooldowns.insert(guild_id, 3600);
        config.record_submission(guild_id, user_id, 1000);
        
        // 10 seconds later
        let remaining = config.check_cooldown(guild_id, user_id, 1010);
        assert_eq!(remaining, Some(3590));
    }

    #[test]
    fn test_cooldown_check_after_cooldown() {
        let mut config = Config {
            discord_token: "test".to_string(),
            confession_threads: HashMap::new(),
            cooldowns: HashMap::new(),
            user_cooldowns: HashMap::new(),
        };
        
        let guild_id = GuildId::new(12345);
        let user_id = UserId::new(67890);
        config.cooldowns.insert(guild_id, 3600);
        config.record_submission(guild_id, user_id, 1000);
        
        // 3600 seconds later (cooldown expired)
        let remaining = config.check_cooldown(guild_id, user_id, 4600);
        assert_eq!(remaining, None);
    }

    #[test]
    fn test_zero_cooldown_disabled() {
        let mut config = Config {
            discord_token: "test".to_string(),
            confession_threads: HashMap::new(),
            cooldowns: HashMap::new(),
            user_cooldowns: HashMap::new(),
        };
        
        let guild_id = GuildId::new(12345);
        let user_id = UserId::new(67890);
        config.cooldowns.insert(guild_id, 0);
        config.record_submission(guild_id, user_id, 1000);
        
        // Immediately after submission with 0 cooldown
        let remaining = config.check_cooldown(guild_id, user_id, 1000);
        assert_eq!(remaining, None);
    }

    #[test]
    fn test_record_submission() {
        let mut config = Config {
            discord_token: "test".to_string(),
            confession_threads: HashMap::new(),
            cooldowns: HashMap::new(),
            user_cooldowns: HashMap::new(),
        };
        
        let guild_id = GuildId::new(12345);
        let user_id = UserId::new(67890);
        let timestamp = 1000;
        
        config.record_submission(guild_id, user_id, timestamp);
        assert_eq!(config.user_cooldowns.get(&(guild_id, user_id)), Some(&timestamp));
    }
}