use std::path::Path;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use poise::serenity_prelude::{ChannelId, GuildId};
use tokio::fs;

const CONFIG_FILE: &str = "config.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub discord_token: String,
    // Map of GuildId -> ChannelId (the thread where new confession threads are created)
    pub confession_threads: HashMap<GuildId, ChannelId>,
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
            };
            default_config.save().await?;
            
            eprintln!("Created default {}. Please fill in your bot token.", CONFIG_FILE);
            return Err("Configuration file created. Please update it and restart.".into());
        }

        let content = fs::read_to_string(path).await?;
        let config: Config = serde_json::from_str(&content)?;
        
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
}