use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::sync::Arc;

use crate::{Data, Error, config::Config, logging::log_confession, utils::{ConfessionModal, ReplyModal, REPLY_BUTTON_ID}};
use poise::{
    ApplicationContext, CreateReply, Modal,
    serenity_prelude::{
        self as serenity, AutoArchiveDuration, CacheHttp, Context, CreateEmbed, CreateEmbedFooter,
        CreateForumPost, CreateInteractionResponse, CreateInteractionResponseMessage,
        CreateMessage, CreateThread, GuildId, Mentionable, ModalInteraction,
    },
};
use serenity::{ButtonStyle, ChannelType, Color};
use tokio::sync::RwLock;

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

    // 2. Send the confession using the shared logic
    let reply = send_confession_logic(
        ctx.guild_id()
            .expect("Guild ID should be present in guild-only command"),
        &ctx.author(),
        ctx.data.config.clone(),
        &ctx.http(),
        confession_content,
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

    let embed = CreateEmbed::new()
        .title("Anonymous Confession")
        .description(confession_content)
        .color(Color::from_rgb(255, 165, 0)) // Orange color
        .footer(CreateEmbedFooter::new("Confessions"));

    // Create the reply button
    let reply_button = serenity::CreateButton::new(REPLY_BUTTON_ID)
        .label("Reply Anonymously")
        .style(ButtonStyle::Primary);

    let message = CreateMessage::new()
        .embed(embed)
        .components(vec![serenity::CreateActionRow::Buttons(vec![reply_button])]);

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
                .send_message(cache, message.clone())
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
                    CreateForumPost::new(thread_name, message)
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
    let reply = send_confession_logic(
        interaction
            .guild_id
            .expect("Guild ID should be present in guild-only command"),
        &interaction.user,
        config,
        ctx.http(),
        confession_content,
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

/// Handles the anonymous reply submission when triggered by the reply button.
pub async fn handle_reply_submission<'a>(
    ctx: &Context,
    interaction: &ModalInteraction,
    data: ReplyModal,
) -> Result<(), Error> {
    let reply_content = data.content.trim().to_string();
    
    // Get the channel ID where the reply button was clicked (the thread/forum post)
    let thread_id = interaction.channel_id;
    
    // Log the reply for auditing (using hash of user ID for anonymity)
    let hash = format!("{:x}", Sha256::digest(&interaction.user.id.to_string()));
    log::warn!(
        "Anonymous reply received: {} | Thread: {} | {}",
        hash,
        thread_id,
        reply_content.replace('\n', " \\n ")
    );
    
    // Create an anonymous reply embed
    let embed = CreateEmbed::new()
        .description(reply_content)
        .color(Color::from_rgb(100, 149, 237)) // Cornflower blue
        .footer(CreateEmbedFooter::new("Anonymous Reply"));
    
    // Send the reply to the thread
    let result = thread_id
        .send_message(
            ctx.http(),
            CreateMessage::new().embed(embed),
        )
        .await;
    
    let response_message = match result {
        Ok(_) => "Your anonymous reply has been posted successfully!".to_string(),
        Err(e) => {
            log::error!(
                "Failed to send anonymous reply in thread {}: {:?}",
                thread_id,
                e
            );
            "An error occurred while posting your reply. Please try again later.".to_string()
        }
    };
    
    // Respond to the interaction
    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content(response_message)
                    .ephemeral(true),
            ),
        )
        .await?;
    
    Ok(())
}
