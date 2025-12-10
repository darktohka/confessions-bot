use chrono::{DateTime, Utc};
use poise::serenity_prelude::{
    self as serenity, AutoArchiveDuration, CreateEmbed, CreateEmbedFooter,
    CreateForumPost, CreateMessage, CreateThread, Mentionable,
};
use serenity::{ChannelType, Color};

use crate::{Context, Error};

/// Manage pending confessions that were flagged by the blacklist.
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "MANAGE_MESSAGES",
    subcommands("list", "approve", "reject"),
    description_localized(
        "en-US",
        "Manage pending confessions that were flagged by the blacklist."
    )
)]
pub async fn review_confession(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// List all pending confessions waiting for review.
#[poise::command(slash_command, guild_only)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be run in a guild.")?;

    let data = ctx.data();
    let config_lock = data.config.clone();
    let config = config_lock.read().await;

    let pending: Vec<_> = config
        .pending_confessions
        .values()
        .filter(|pc| pc.guild_id == guild_id)
        .collect();

    if pending.is_empty() {
        ctx.say("There are no pending confessions awaiting review.")
            .await?;
        return Ok(());
    }

    let mut response = format!("**Pending Confessions ({} total):**\n\n", pending.len());
    
    for confession in pending {
        let timestamp = DateTime::from_timestamp(confession.timestamp, 0)
            .unwrap_or_else(|| Utc::now());
        
        response.push_str(&format!(
            "**ID:** `{}`\n**Submitted:** {}\n**Flagged Terms:** {}\n**Content Preview:** {}\n\n---\n\n",
            confession.confession_id,
            timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            confession.flagged_terms.join(", "),
            if confession.content.len() > 100 {
                format!("{}...", &confession.content[..100])
            } else {
                confession.content.clone()
            }
        ));
    }

    ctx.say(response).await?;
    Ok(())
}

/// Approve a pending confession and post it.
#[poise::command(slash_command, guild_only)]
pub async fn approve(
    ctx: Context<'_>,
    #[description = "The ID of the confession to approve"] confession_id: String,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be run in a guild.")?;

    let data = ctx.data();
    let config_lock = data.config.clone();
    
    // Get the pending confession and remove it from the queue
    let pending_confession = {
        let mut config = config_lock.write().await;
        config.pending_confessions.remove(&confession_id)
    };

    let pending_confession = match pending_confession {
        Some(pc) if pc.guild_id == guild_id => pc,
        Some(_) => {
            ctx.say("That confession ID belongs to a different guild.")
                .await?;
            return Ok(());
        }
        None => {
            ctx.say(format!("No pending confession found with ID: {}", confession_id))
                .await?;
            return Ok(());
        }
    };

    // Get the target channel
    let target_channel_id = {
        let config = config_lock.read().await;
        match config.confession_threads.get(&guild_id) {
            Some(id) => *id,
            None => {
                ctx.say("The confession channel has not been set up for this guild.")
                    .await?;
                return Ok(());
            }
        }
    };

    // Post the confession
    let channel = match target_channel_id.to_channel(ctx.http()).await {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to fetch channel {}: {:?}", target_channel_id, e);
            ctx.say("Failed to fetch the target channel.").await?;
            return Ok(());
        }
    };

    let channel_kind = match channel {
        serenity::Channel::Guild(guild_channel) => guild_channel.kind,
        _ => {
            ctx.say("The configured confession channel is not a guild channel.")
                .await?;
            return Ok(());
        }
    };

    let now: DateTime<Utc> = Utc::now();
    let thread_name = format!("Confession - {}", now.format("%Y-%m-%d %H:%M:%S UTC"));

    let embed = CreateEmbed::new()
        .title("Anonymous Confession")
        .description(&pending_confession.content)
        .color(Color::from_rgb(255, 165, 0))
        .footer(CreateEmbedFooter::new("Confessions"));

    let result = match channel_kind {
        ChannelType::Text | ChannelType::PublicThread | ChannelType::PrivateThread => {
            let new_thread = target_channel_id
                .create_thread(
                    ctx.http(),
                    CreateThread::new(thread_name)
                        .kind(ChannelType::PublicThread)
                        .auto_archive_duration(AutoArchiveDuration::ThreeDays),
                )
                .await?;

            new_thread
                .send_message(ctx.http(), CreateMessage::new().embed(embed))
                .await?;

            format!("in {}", new_thread.id.mention())
        }
        ChannelType::Forum => {
            let thread = target_channel_id
                .create_forum_post(
                    ctx.http(),
                    CreateForumPost::new(thread_name, CreateMessage::new().embed(embed))
                        .auto_archive_duration(AutoArchiveDuration::ThreeDays),
                )
                .await?;

            format!("in {}", thread.id.mention())
        }
        _ => {
            ctx.say("The configured confession channel is not a supported type.")
                .await?;
            return Ok(());
        }
    };

    // Save the updated configuration (confession removed from pending)
    {
        let config = config_lock.read().await;
        if let Err(e) = config.save().await {
            log::error!("Failed to save configuration: {:?}", e);
        }
    }

    ctx.say(format!(
        "Confession `{}` has been approved and posted {}.",
        confession_id, result
    ))
    .await?;

    Ok(())
}

/// Reject a pending confession and remove it from the queue.
#[poise::command(slash_command, guild_only)]
pub async fn reject(
    ctx: Context<'_>,
    #[description = "The ID of the confession to reject"] confession_id: String,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be run in a guild.")?;

    let data = ctx.data();
    let config_lock = data.config.clone();
    
    // Remove the pending confession
    let removed = {
        let mut config = config_lock.write().await;
        let pending = config.pending_confessions.remove(&confession_id);
        
        match pending {
            Some(pc) if pc.guild_id == guild_id => {
                // Save the configuration
                if let Err(e) = config.save().await {
                    log::error!("Failed to save configuration: {:?}", e);
                }
                true
            }
            Some(pc) => {
                // Put it back if it's from a different guild
                config.pending_confessions.insert(confession_id.clone(), pc);
                false
            }
            None => false,
        }
    };

    if removed {
        ctx.say(format!(
            "Confession `{}` has been rejected and removed from the review queue.",
            confession_id
        ))
        .await?;
    } else {
        ctx.say(format!(
            "No pending confession found with ID: {}",
            confession_id
        ))
        .await?;
    }

    Ok(())
}
