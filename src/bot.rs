use anyhow::Context as AnyhowContext;
use poise::{Framework, serenity_prelude as serenity};

use crate::linkding::LinkdingApi;

struct BotData {
    linkding: LinkdingApi,
}

/// Register this bot's slash commands with Discord.
///
/// This command is used to register the bot's commands when it starts up.
#[poise::command(prefix_command)]
pub async fn register(ctx: Context<'_>) -> anyhow::Result<()> {
    poise::builtins::register_application_commands_buttons(ctx)
        .await
        .context("Could not spawn message to register Discord slash commands")?;
    Ok(())
}

type Context<'a> = poise::Context<'a, BotData, anyhow::Error>;

/// Determine if the bot is working and can reach the Linkding instance.
#[poise::command(slash_command, prefix_command)]
async fn status(context: Context<'_>) -> anyhow::Result<()> {
    let bot = context.framework().user_data().await;
    let reply = match bot.linkding.is_reachable().await {
        Ok(()) => format!("✅ [Linkding]({}) is reachable!", bot.linkding.base_url()),
        Err(e) => format!(
            "❌ [Linkding]({}) is not reachable: {:?}",
            bot.linkding.base_url(),
            e
        ),
    };
    context.reply(reply).await?;

    Ok(())
}

/// Create a new Discord client that uses the bot's commands.
pub async fn create_client(token: String) -> anyhow::Result<serenity::Client> {
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = Framework::<BotData, anyhow::Error>::builder()
        .options(poise::FrameworkOptions {
            commands: vec![register(), status()],
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands)
                    .await
                    .context("Could not globally register Discord sllash commands")?;

                let data = BotData {
                    linkding: LinkdingApi::from_env()
                        .context("Could not create Linkding client")?,
                };

                tracing::info!(
                    name = ready.user.name,
                    id = ready.user.id.get(),
                    "Discord client connected"
                );
                Ok(data)
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await?;
    Ok(client)
}
