use poise::serenity_prelude::{self as serenity, CreateEmbed, CreateMessage};
use serenity::{ButtonStyle, Color};

use crate::{Context, Error, utils::CONFESS_BUTTON_ID};

/// Creates a message with a button to submit an anonymous confession.
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "MANAGE_MESSAGES",
    description_localized(
        "en-US",
        "Sends the message with the anonymous confession button into the current channel."
    )
)]
pub async fn confessembed(ctx: Context<'_>) -> Result<(), Error> {
    let message = CreateMessage::default()
        .embed(CreateEmbed::new().title("Anonymous Confessions")
                .description("Click the button below to submit an anonymous confession.\nA new thread will be created for each submission.\n\n**Note:** All confessions are anonymous and cannot be traced back to you.")
                .color(Color::ORANGE))
        .components(vec![serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(CONFESS_BUTTON_ID)
                .label("Submit Anonymous Confession")
                .style(ButtonStyle::Primary),
        ])]);

    ctx.channel_id().send_message(ctx.http(), message).await?;
    ctx.reply("Confession embed sent successfully!").await?;

    Ok(())
}
