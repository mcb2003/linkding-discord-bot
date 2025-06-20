use anyhow::Context as AnyhowContext;
use poise::{Framework, serenity_prelude as serenity};
use reqwest::Url;

use crate::linkding::LinkdingApi;

struct BotData {
    linkding: LinkdingApi,
}

/// Show this menu
#[tracing::instrument(skip(ctx))]
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> anyhow::Result<()> {
    let config = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: "\
Type ?help command for more info on a command.
You can edit your message to the bot and the bot will edit its response.",
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

/// Register this bot's slash commands with Discord.
///
/// This command is used to register the bot's commands when it starts up.
#[tracing::instrument(skip(ctx))]
#[poise::command(prefix_command)]
pub async fn register(ctx: Context<'_>) -> anyhow::Result<()> {
    poise::builtins::register_application_commands_buttons(ctx)
        .await
        .context("Could not spawn message to register Discord slash commands")?;
    Ok(())
}

type Context<'a> = poise::Context<'a, BotData, anyhow::Error>;

/// Determine if the bot is working and can reach the Linkding instance.
#[tracing::instrument(skip(ctx))]
#[poise::command(slash_command, prefix_command)]
async fn status(ctx: Context<'_>) -> anyhow::Result<()> {
    let bot = ctx.framework().user_data().await;
    let reply = match bot.linkding.is_reachable().await {
        Ok(()) => format!("✅ [Linkding]({}) is reachable!", bot.linkding.base_url()),
        Err(e) => format!(
            "❌ [Linkding]({}) is not reachable: {:?}",
            bot.linkding.base_url(),
            e
        ),
    };
    ctx.reply(reply).await?;

    Ok(())
}

/// Create a new Discord client that uses the bot's commands.
#[tracing::instrument(skip(token))]
pub async fn create_client(token: String) -> anyhow::Result<serenity::Client> {
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = Framework::<BotData, anyhow::Error>::builder()
        .options(poise::FrameworkOptions {
            commands: vec![help(), register(), status()],
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

pub type DateTime = chrono::DateTime<chrono::Utc>;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Bookmark {
    id: u64,
    url: Url,
    title: String,
    description: String,
    notes: String,
    web_archive_snapshot_url: Url,
    favicon_url: String,
    preview_image_url: String,
    is_archived: bool,
    unread: bool,
    shared: bool,
    tag_names: Vec<String>,
    date_added: DateTime,
    date_modified: DateTime,
}
