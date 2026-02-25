use crate::{Context, Error};
use poise::serenity_prelude::{self as serenity, CreateEmbed};
use serenity::Color;

/// Shows statistics about confession button usage.
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "MANAGE_MESSAGES",
    description_localized(
        "en-US",
        "Shows how many times each user has pressed the confession button."
    )
)]
pub async fn buttonstats(ctx: Context<'_>) -> Result<(), Error> {
    let stats = ctx.data().button_stats.read().await;
    
    if stats.press_counts.is_empty() {
        ctx.say("No button presses have been recorded yet.").await?;
        return Ok(());
    }

    // Sort users by press count (descending)
    let mut sorted_stats: Vec<_> = stats.press_counts.iter().collect();
    sorted_stats.sort_by(|a, b| b.1.cmp(a.1));

    // Build the description with user mentions and counts
    let mut description = String::new();
    let total_presses: u64 = stats.press_counts.values().sum();
    
    for (user_id, count) in sorted_stats.iter().take(25) { // Discord embed field limit
        description.push_str(&format!("<@{}> - {} press{}\n", user_id, count, if **count == 1 { "" } else { "es" }));
    }

    if sorted_stats.len() > 25 {
        description.push_str(&format!("\n...and {} more user{}", 
            sorted_stats.len() - 25,
            if sorted_stats.len() - 25 == 1 { "" } else { "s" }));
    }

    let embed = CreateEmbed::new()
        .title("Confession Button Statistics")
        .description(description)
        .color(Color::BLUE)
        .footer(serenity::CreateEmbedFooter::new(format!(
            "Total: {} user{}, {} total press{}",
            stats.press_counts.len(),
            if stats.press_counts.len() == 1 { "" } else { "s" },
            total_presses,
            if total_presses == 1 { "" } else { "es" }
        )));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
