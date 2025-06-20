use anyhow::Context as AnyhowContext;
use poise::{Framework, serenity_prelude as serenity};

use crate::linkding_api::LinkdingApi;

struct BotData {
    linkding: LinkdingApi,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, BotData, Error>;

/// Create a new Discord client that uses the bot's commands.
pub async fn create_client(token: String) -> anyhow::Result<serenity::Client> {
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = Framework::<BotData, anyhow::Error>::builder()
        .options(poise::FrameworkOptions {
            commands: vec![],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(BotData {
                    linkding: LinkdingApi::from_env()
                        .context("Could not create Linkding client")?,
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;
    Ok(client)
}
