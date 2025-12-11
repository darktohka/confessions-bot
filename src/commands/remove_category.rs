use crate::{Context, Error};

/// Remove a category from the available confession categories in this guild.
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "MANAGE_MESSAGES",
    description_localized(
        "en-US",
        "Remove a category/tag from the available confession categories."
    )
)]
pub async fn remove_category(
    ctx: Context<'_>,
    #[description = "The name of the category to remove"]
    #[max_length = 50]
    category_name: String,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be run in a guild.")?;

    let category_name = category_name.trim().to_string();

    if category_name.is_empty() {
        ctx.say("Error: Category name cannot be empty.").await?;
        return Ok(());
    }

    let data = ctx.data();
    let config_lock = data.config.clone();
    let mut config = config_lock.write().await;

    // Get the categories vector for this guild
    let categories = match config.categories.get_mut(&guild_id) {
        Some(cats) => cats,
        None => {
            ctx.say("Error: No categories have been configured for this guild yet.").await?;
            return Ok(());
        }
    };

    // Find and remove the category (case-insensitive)
    let original_len = categories.len();
    categories.retain(|cat| !cat.eq_ignore_ascii_case(&category_name));

    if categories.len() == original_len {
        ctx.say(format!("Error: Category '{}' was not found.", category_name)).await?;
        return Ok(());
    }

    // Clone the categories list for the success message
    let categories_list = if categories.is_empty() {
        "None".to_string()
    } else {
        categories.join(", ")
    };

    // Save the updated configuration
    if let Err(e) = config.save().await {
        log::error!("Failed to save configuration: {:?}", e);
        let error_msg = format!(
            "Successfully removed category '{}' but failed to save the configuration. \
             Please contact a server administrator.",
            category_name
        );
        ctx.say(error_msg).await?;
        return Ok(());
    }

    ctx.say(format!(
        "Successfully removed category '{}'.\nRemaining categories: {}",
        category_name,
        categories_list
    )).await?;

    Ok(())
}
