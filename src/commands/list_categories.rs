use crate::{Context, Error};

/// List all available categories for confessions in this guild.
#[poise::command(
    slash_command,
    guild_only,
    description_localized(
        "en-US",
        "List all available categories/tags for confessions in this guild."
    )
)]
pub async fn list_categories(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be run in a guild.")?;

    let data = ctx.data();
    let config_lock = data.config.clone();
    let config = config_lock.read().await;

    match config.categories.get(&guild_id) {
        Some(categories) if !categories.is_empty() => {
            let category_list = categories
                .iter()
                .map(|c| format!("• {}", c))
                .collect::<Vec<_>>()
                .join("\n");
            ctx.say(format!(
                "**Available categories for confessions:**\n{}",
                category_list
            )).await?;
        }
        _ => {
            ctx.say("No categories have been configured for this guild yet.\nAdmins can add categories using `/add_category`.").await?;
        }
    }

    Ok(())
}
