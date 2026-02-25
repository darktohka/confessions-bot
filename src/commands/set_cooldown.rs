use crate::{Context, Error};

/// Set the cooldown period (in seconds) between confession submissions for this guild.
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "MANAGE_MESSAGES",
    description_localized(
        "en-US",
        "Set the cooldown period (in seconds) between confession submissions."
    )
)]
pub async fn set_cooldown(
    ctx: Context<'_>,
    #[description = "Cooldown period in seconds (e.g., 3600 for 1 hour, 0 to disable)"]
    #[min = 0]
    seconds: u64,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be run in a guild.")?;

    let data = ctx.data();
    let config_lock = data.config.clone();

    let mut config = config_lock.write().await;
    config.cooldowns.insert(guild_id, seconds);

    // Save the updated configuration
    if let Err(e) = config.save().await {
        log::error!("Failed to save configuration: {:?}", e);
        ctx.say(format!("Successfully set the cooldown to {} seconds but failed to save configuration: {:?}", seconds, e)).await?;
        return Ok(());
    }

    if seconds == 0 {
        ctx.say("Successfully disabled the confession cooldown for this guild. Users can now submit confessions without waiting.").await?;
    } else {
        let minutes = seconds / 60;
        let remaining_seconds = seconds % 60;
        
        if minutes > 0 && remaining_seconds > 0 {
            ctx.say(format!(
                "Successfully set the confession cooldown to {} minute{} and {} second{}.",
                minutes,
                if minutes == 1 { "" } else { "s" },
                remaining_seconds,
                if remaining_seconds == 1 { "" } else { "s" }
            )).await?;
        } else if minutes > 0 {
            ctx.say(format!(
                "Successfully set the confession cooldown to {} minute{}.",
                minutes,
                if minutes == 1 { "" } else { "s" }
            )).await?;
        } else {
            ctx.say(format!(
                "Successfully set the confession cooldown to {} second{}.",
                seconds,
                if seconds == 1 { "" } else { "s" }
            )).await?;
        }
    }

    Ok(())
}
