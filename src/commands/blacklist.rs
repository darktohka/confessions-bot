use crate::{Context, Error};

/// Manage the confession blacklist (add, remove, or list blacklisted words/phrases).
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "MANAGE_MESSAGES",
    subcommands("add", "remove", "list"),
    description_localized(
        "en-US",
        "Manage the confession blacklist (add, remove, or list blacklisted words/phrases)."
    )
)]
pub async fn blacklist(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Add a word or phrase to the blacklist.
#[poise::command(slash_command, guild_only)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "The word or phrase to blacklist (case-insensitive)"] term: String,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be run in a guild.")?;

    let data = ctx.data();
    let config_lock = data.config.clone();
    let mut config = config_lock.write().await;

    let blacklist = config.blacklist.entry(guild_id).or_insert_with(Vec::new);
    
    let term_lower = term.to_lowercase();
    
    // Check if term already exists (case-insensitive)
    if blacklist.iter().any(|t| t.to_lowercase() == term_lower) {
        ctx.say(format!("The term \"{}\" is already in the blacklist.", term))
            .await?;
        return Ok(());
    }

    blacklist.push(term.clone());

    // Save the updated configuration
    if let Err(e) = config.save().await {
        log::error!("Failed to save configuration: {:?}", e);
        ctx.say(format!(
            "Added \"{}\" to the blacklist but failed to save configuration: {:?}",
            term, e
        ))
        .await?;
        return Ok(());
    }

    ctx.say(format!(
        "Successfully added \"{}\" to the blacklist. Confessions containing this term will be flagged for review.",
        term
    ))
    .await?;

    Ok(())
}

/// Remove a word or phrase from the blacklist.
#[poise::command(slash_command, guild_only)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "The word or phrase to remove from the blacklist"] term: String,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be run in a guild.")?;

    let data = ctx.data();
    let config_lock = data.config.clone();
    let mut config = config_lock.write().await;

    let blacklist = config.blacklist.entry(guild_id).or_insert_with(Vec::new);
    
    let term_lower = term.to_lowercase();
    let initial_len = blacklist.len();
    
    // Remove all case-insensitive matches
    blacklist.retain(|t| t.to_lowercase() != term_lower);

    if blacklist.len() == initial_len {
        ctx.say(format!("The term \"{}\" was not found in the blacklist.", term))
            .await?;
        return Ok(());
    }

    // Save the updated configuration
    if let Err(e) = config.save().await {
        log::error!("Failed to save configuration: {:?}", e);
        ctx.say(format!(
            "Removed \"{}\" from the blacklist but failed to save configuration: {:?}",
            term, e
        ))
        .await?;
        return Ok(());
    }

    ctx.say(format!(
        "Successfully removed \"{}\" from the blacklist.",
        term
    ))
    .await?;

    Ok(())
}

/// List all blacklisted words and phrases.
#[poise::command(slash_command, guild_only)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or("This command must be run in a guild.")?;

    let data = ctx.data();
    let config_lock = data.config.clone();
    let config = config_lock.read().await;

    let blacklist = config.blacklist.get(&guild_id);

    match blacklist {
        Some(list) if !list.is_empty() => {
            let terms = list
                .iter()
                .enumerate()
                .map(|(i, term)| format!("{}. \"{}\"", i + 1, term))
                .collect::<Vec<_>>()
                .join("\n");
            
            ctx.say(format!("**Blacklisted terms ({} total):**\n{}", list.len(), terms))
                .await?;
        }
        _ => {
            ctx.say("There are no blacklisted terms for this guild.")
                .await?;
        }
    }

    Ok(())
}
