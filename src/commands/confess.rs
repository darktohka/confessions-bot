use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::{Data, Error, config::Config, logging::log_confession, utils::ConfessionModal};
use poise::{
    ApplicationContext, CreateReply, Modal,
    serenity_prelude::{
        self as serenity, AutoArchiveDuration, CacheHttp, Context, CreateEmbed, CreateEmbedFooter,
        CreateForumPost, CreateInteractionResponse, CreateInteractionResponseMessage,
        CreateMessage, CreateThread, GuildId, Mentionable, ModalInteraction,
    },
};
use serenity::{ChannelType, Color};
use tokio::sync::RwLock;

/// Helper function to process categories from modal data
fn process_categories(categories: Option<String>) -> Option<String> {
    categories
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Submit an anonymous confession.
#[poise::command(
    slash_command,
    guild_only,
    description_localized(
        "en-US",
        "Submit an anonymous confession (all submissions are anonymous)."
    )
)]
pub async fn confess(ctx: ApplicationContext<'_, Data, Error>) -> Result<(), Error> {
    // 1. Execute the modal and wait for submission
    let data = match ConfessionModal::execute(ctx).await {
        Ok(Some(data)) => data,
        Ok(None) => {
            // User cancelled the modal
            return Ok(());
        }
        Err(e) => {
            log::error!("Error executing modal: {:?}", e);
            ctx.say("An error occurred while processing the confession modal.")
                .await?;
            return Err(Box::new(e));
        }
    };
    let confession_content = data.content.trim().to_string();
    let categories = process_categories(data.categories);

    // 2. Send the confession using the shared logic
    let reply = send_confession_logic(
        ctx.guild_id()
            .expect("Guild ID should be present in guild-only command"),
        &ctx.author(),
        ctx.data.config.clone(),
        &ctx.http(),
        confession_content,
        categories,
    )
    .await;

    // 3. Send the reply to the user
    ctx.send(CreateReply::default().content(reply).ephemeral(true))
        .await?;
    Ok(())
}

// Generic logic function to handle the core logic of sending a confession
async fn send_confession_logic<'a>(
    guild_id: GuildId,
    author: &'a serenity::User,
    config: Arc<RwLock<Config>>,
    cache: &'a serenity::Http,
    confession_content: String,
    categories: Option<String>,
) -> String {
    // 1. Log the confession for auditing
    // Use a hash of the author's ID to maintain anonymity
    // This allows tracking of multiple requests from the same user without revealing their identity
    // in case they abuse the system in any way
    let hash = format!("{:x}", Sha256::digest(&author.id.to_string()));
    log_confession(&hash, &confession_content);

    // 2. Get the target channel ID and type from configuration
    let target_channel_id = {
        let config = config.read().await;

        match config.confession_threads.get(&guild_id) {
            Some(id) => *id,
            None => {
                return "The confession channel has not been set up for this guild. Please ask a staff member to use `/set_confession_thread`.".to_string();
            }
        }
    };

    // Fetch channel type
    let channel = match target_channel_id.to_channel(cache).await {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to fetch channel {}: {:?}", target_channel_id, e);
            return "An error occurred while fetching the target channel information. Please try again later.".to_string();
        }
    };

    let channel_kind = match channel {
        serenity::Channel::Guild(guild_channel) => guild_channel.kind,
        _ => {
            log::error!(
                "Target channel {} is not a guild channel.",
                target_channel_id
            );
            return "The configured confession channel is not a guild channel.".to_string();
        }
    };

    // Prepare common elements
    let now: DateTime<Utc> = Utc::now();
    let thread_name = format!("Confession - {}", now.format("%Y-%m-%d %H:%M:%S UTC"));

    // Build the embed with optional categories
    let mut embed = CreateEmbed::new()
        .title("Anonymous Confession")
        .description(confession_content)
        .color(Color::from_rgb(255, 165, 0)) // Orange color
        .footer(CreateEmbedFooter::new("Confessions"));

    // Add categories field if provided
    if let Some(ref cats) = categories {
        if !cats.is_empty() {
            embed = embed.field("Categories", cats, false);
        }
    }

    // 3. Create a new thread/post inside the target channel
    let _new_thread_id = match channel_kind {
        ChannelType::Text | ChannelType::PublicThread | ChannelType::PrivateThread => {
            // Create a thread in a Text channel or a sub-thread in an existing thread
            let new_thread = match target_channel_id
                .create_thread(
                    cache,
                    CreateThread::new(thread_name)
                        .kind(ChannelType::PublicThread)
                        .auto_archive_duration(AutoArchiveDuration::ThreeDays),
                )
                .await
            {
                Ok(thread) => thread,
                Err(e) => {
                    log::error!(
                        "Failed to create thread in channel {}: {:?}",
                        target_channel_id,
                        e
                    );
                    return "An error occurred while creating a thread for your confession. Please try again later.".to_string();
                }
            };

            // 4. Send the anonymous confession embed to the new thread
            match new_thread
                .send_message(cache, CreateMessage::new().embed(embed))
                .await
            {
                Ok(_) => new_thread.id,
                Err(e) => {
                    log::error!(
                        "Failed to send confession message in thread {}: {:?}",
                        new_thread.id,
                        e
                    );
                    return "An error occurred while sending your confession. Please try again later."
                        .to_string();
                }
            }
        }
        ChannelType::Forum => {
            // Create a post in a Forum channel
            match target_channel_id
                .create_forum_post(
                    cache,
                    CreateForumPost::new(thread_name, CreateMessage::new().embed(embed))
                        .auto_archive_duration(AutoArchiveDuration::ThreeDays),
                )
                .await
            {
                Ok(thread) => thread.id,
                Err(e) => {
                    log::error!(
                        "Failed to create forum post in channel {}: {:?}",
                        target_channel_id,
                        e
                    );
                    return "An error occurred while creating a forum post for your confession. Please try again later.".to_string();
                }
            }
        }
        _ => {
            log::error!(
                "Target channel {} is not a supported type: {:?}",
                target_channel_id,
                channel_kind
            );
            return "The configured confession channel is not a supported type (Text, Forum, or Thread).".to_string();
        }
    };

    // 5. Acknowledge the submission
    format!(
        "Your anonymous confession has been submitted! See the new post/thread in {}.",
        target_channel_id.mention()
    )
}

/// Handles the modal submission when triggered by the button interaction.
/// This function is called by the interaction handler in the next step.
pub async fn handle_modal_submission<'a>(
    ctx: &Context,
    config: Arc<RwLock<Config>>,
    interaction: &ModalInteraction,
    data: ConfessionModal,
) -> Result<(), Error> {
    let confession_content = data.content.trim().to_string();
    let categories = process_categories(data.categories);

    let reply = send_confession_logic(
        interaction
            .guild_id
            .expect("Guild ID should be present in guild-only command"),
        &interaction.user,
        config,
        ctx.http(),
        confession_content,
        categories,
    )
    .await;

    // Respond to the interaction with the reply
    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(reply)
                    .ephemeral(true),
            ),
        )
        .await?;

    Ok(())
}
