use crate::{Context, Error};

/// Add a category for confessions in this guild.
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "MANAGE_MESSAGES",
    description_localized(
        "en-US",
        "Add a category/tag that users can select when submitting confessions."
    )
)]
pub async fn add_category(
    ctx: Context<'_>,
    #[description = "The name of the category to add (e.g., 'Funny', 'Serious', 'Advice Needed')"]
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

    // Get or create the categories vector for this guild
    let categories = config.categories.entry(guild_id).or_insert_with(Vec::new);

    // Check if category already exists (case-insensitive)
    if categories.iter().any(|cat| cat.eq_ignore_ascii_case(&category_name)) {
        ctx.say(format!("Error: Category '{}' already exists for this guild.", category_name)).await?;
        return Ok(());
    }

    // Add the category
    categories.push(category_name.clone());

    // Clone the categories list for the success message
    let categories_list = categories.join(", ");

    // Save the updated configuration
    if let Err(e) = config.save().await {
        log::error!("Failed to save configuration: {:?}", e);
        ctx.say(format!("Successfully added category '{}' but failed to save configuration: {:?}", category_name, e)).await?;
        return Ok(());
    }

    ctx.say(format!(
        "Successfully added category '{}' for this guild.\nCurrent categories: {}",
        category_name,
        categories_list
    )).await?;

    Ok(())
}
