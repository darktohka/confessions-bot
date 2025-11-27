use poise::serenity_prelude::{self as serenity, ChannelId, Mentionable};

use crate::{Context, Error};

/// Choose the guild channel (Text or Forum) where all confession threads/posts will be created.
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "MANAGE_MESSAGES",
    description_localized(
        "en-US",
        "Choose the guild channel (Text or Forum) where all confession threads/posts will be created."
    )
)]
pub async fn set_confession_thread(
    ctx: Context<'_>,
    #[description = "The channel (Text or Forum) where new confession threads/posts should be created."]
    thread_channel: ChannelId,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be run in a guild.")?;

    // Check if the provided channel is a thread (or a channel that supports threads)
    let channel = thread_channel.to_channel(ctx.http()).await?;

    if let serenity::Channel::Guild(guild_channel) = channel {
        if !matches!(
            guild_channel.kind,
            serenity::ChannelType::Text | serenity::ChannelType::Forum
        ) {
            ctx.say("Error: The provided channel must be a Text channel or a Forum channel.")
                .await?;
            return Ok(());
        }
    } else {
        ctx.say("Error: The provided channel must be a guild channel.")
            .await?;
        return Ok(());
    }

    let data = ctx.data();
    let config_lock = data.config.clone();

    let mut config = config_lock.write().await;
    config.confession_threads.insert(guild_id, thread_channel);

    // Save the updated configuration
    if let Err(e) = config.save().await {
        log::error!("Failed to save configuration: {:?}", e);
        ctx.say(format!("Successfully set the confession thread channel to {} but failed to save configuration: {:?}", thread_channel.mention(), e)).await?;
        return Ok(());
    }

    ctx.say(format!(
        "Successfully set the confession channel for this guild to {}. New confessions will be created as threads/posts in this channel.",
        thread_channel.mention()
    )).await?;

    Ok(())
}
