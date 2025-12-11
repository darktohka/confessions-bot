mod commands;
mod config;
mod logging;
mod utils;

use log::LevelFilter;
use log4rs::{
    append::{
        console::ConsoleAppender,
        rolling_file::{
            RollingFileAppender,
            policy::compound::{
                CompoundPolicy, roll::fixed_window::FixedWindowRoller, trigger::size::SizeTrigger,
            },
        },
    },
    config::{Appender, Config as Log4rsConfig, Root},
    encode::pattern::PatternEncoder,
};
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;

use poise::{
    Modal,
    serenity_prelude::{self as serenity, CacheHttp, GatewayIntents},
};

use commands::{confess, confessembed, set_confession_thread, add_category};
use config::Config;
use utils::{CONFESS_BUTTON_ID, ConfessionModal};

// --- Poise Types ---

/// User data, which is stored and accessible in all command invocations
pub struct Data {
    pub config: Arc<RwLock<Config>>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

// --- Error Handler ---

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            log::error!("Error in command `{}`: {:?}", ctx.command().name, error);
            if let Err(e) = ctx.say(format!("An error occurred: {}", error)).await {
                log::error!("Failed to send error message: {:?}", e);
            }
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                log::error!("Error while handling error: {:?}", e);
            }
        }
    }
}

// --- Main ---

#[tokio::main]
async fn main() {
    // Configure log4rs for file rotation and stdout logging
    let window_roller = FixedWindowRoller::builder()
        .base(1)
        .build("logs/confessions_audit.{}.log", 10)
        .expect("Failed to build window roller");

    let size_trigger = SizeTrigger::new(10 * 1024 * 1024); // 10MB

    let compound_policy = CompoundPolicy::new(Box::new(size_trigger), Box::new(window_roller));

    let file_appender = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[{d(%Y-%m-%dT%H:%M:%S%z)}] | {l} | {M} | {m}\n",
        )))
        .build("logs/confessions_audit.log", Box::new(compound_policy))
        .expect("Failed to build rolling file appender");

    let stdout_appender = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "[{d(%Y-%m-%dT%H:%M:%S%z)}] | {l} | {M} | {m}\n",
        )))
        .build();

    let log_config = Log4rsConfig::builder()
        .appender(Appender::builder().build("file", Box::new(file_appender)))
        .appender(Appender::builder().build("stdout", Box::new(stdout_appender)))
        .build(
            Root::builder()
                .appender("file")
                .appender("stdout")
                .build(LevelFilter::Warn),
        )
        .expect("Failed to build log4rs config");

    log4rs::init_config(log_config).expect("Failed to initialize log4rs");

    // Load configuration
    let config = match Config::load().await {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to load configuration: {}", e);
            return;
        }
    };

    let token = config.discord_token.clone();
    let config_arc = Arc::new(RwLock::new(config));
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                set_confession_thread::set_confession_thread(),
                confessembed::confessembed(),
                confess::confess(),
                add_category::add_category(),
            ],
            event_handler: |ctx, event, _framework, _data| {
                Box::pin(async move {
                    if let serenity::FullEvent::InteractionCreate { interaction } = event {
                        if let Some(component) = interaction.as_message_component() {
                            if component.data.custom_id == CONFESS_BUTTON_ID {
                                let custom_id = component.id.to_string();
                                component
                                    .create_response(
                                        ctx.http(),
                                        ConfessionModal::create(None, custom_id.clone()),
                                    )
                                    .await?;

                                let response =
                                    serenity::collector::ModalInteractionCollector::new(&ctx.shard)
                                        .filter(move |modal_interaction| {
                                            modal_interaction.data.custom_id == custom_id
                                        })
                                        .timeout(Duration::from_secs(3600))
                                        .await;

                                if let Some(modal_interaction) = response {
                                    let data =
                                        ConfessionModal::parse(modal_interaction.data.clone());

                                    if let Ok(data) = data {
                                        confess::handle_modal_submission(
                                            &ctx,
                                            _data.config.clone(),
                                            &modal_interaction,
                                            data,
                                        )
                                        .await?;
                                    }
                                }
                            }
                        }
                    }
                    Ok(())
                })
            },
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("~".into()),
                ..Default::default()
            },
            on_error: |error| Box::pin(on_error(error)),
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                log::info!("Registering commands globally...");
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data { config: config_arc })
            })
        })
        .build();

    // We need Guilds, MessageContent, and GuildMessages for command registration and interaction handling
    let intents = GatewayIntents::empty(); //serenity::GatewayIntents::non_privileged();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    if let Err(why) = client.unwrap().start().await {
        log::error!("Client error: {:?}", why);
    }
}
