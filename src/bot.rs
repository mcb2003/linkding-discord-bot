use anyhow::Context as AnyhowContext;
use poise::{CreateReply, Framework, serenity_prelude as serenity};
use url::Url;

use crate::linkding::{Bookmark, LinkdingApi};

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
            "❌ [Linkding]({}) is not reachable: {}",
            bot.linkding.base_url(),
            e
        ),
    };
    ctx.reply(reply).await?;

    Ok(())
}

/// Get a link by its ID.
#[tracing::instrument(skip(ctx))]
#[poise::command(slash_command, prefix_command, track_edits)]
async fn link(
    ctx: Context<'_>,
    #[description = "The ID of the link to retrieve"] id: u64,
) -> anyhow::Result<()> {
    let bot = ctx.framework().user_data().await;
    let bookmark = bot.linkding.bookmark(id).await?;

    let embed = bookmark_to_embed(bookmark, bot.linkding.base_url());
    ctx.send(CreateReply::default().reply(true).embed(embed))
        .await?;

    Ok(())
}

/// Convert a [`Bookmark`] into a Discord embed.
fn bookmark_to_embed(bookmark: Bookmark, linkding_base_url: &Url) -> serenity::CreateEmbed {
    let mut embed = serenity::CreateEmbed::default()
        .title(bookmark.title)
        .description(bookmark.description)
        .url(bookmark.url)
        .timestamp(bookmark.date_added);

    let tags = tags_to_md(&bookmark.tag_names, linkding_base_url);
    embed = embed.field("Tags", tags, false);

    if let Some(archive_url) = bookmark.web_archive_snapshot_url {
        let value = format!("[view on the Wayback Machine]({})", archive_url);
        embed = embed.field("Snapshot", value, true);
    }

    if !bookmark.notes.is_empty() {
        embed = embed.field("Notes", bookmark.notes, false);
    }

    // Various miscelaneous information displayed as flags in the footer
    let flags = {
        let mut flags = [""; 3];
        let mut total_flags = 0;
        if bookmark.unread {
            flags[total_flags] = "📥 Unread";
            total_flags += 1;
        }
        if bookmark.shared {
            flags[total_flags] = "🔗 Shared";
            total_flags += 1;
        }
        if bookmark.is_archived {
            flags[total_flags] = "🗃️ Archived";
            total_flags += 1;
        }
        flags[..total_flags].join(" | ")
    };
    if !flags.is_empty() {
        embed = embed.footer(serenity::CreateEmbedFooter::new(flags));
    }

    if let Some(image) = bookmark.preview_image_url {
        embed = embed.image(image);
    }
    if let Some(icon) = bookmark.favicon_url {
        embed = embed.thumbnail(icon);
    }

    embed
}

/// Stringify a list of Linkding tags as markdown with embedded links.
///
/// We ensure the output is never longer than 1024 characters, the maximum length you can use in a Discord embed field.
fn tags_to_md(tags: &[impl AsRef<str>], base_url: &Url) -> String {
    use std::fmt::Write;

    let tag_url = base_url.join("bookmarks").expect("Invalid tag URL");
    let tag_page = |tag: &str| {
        let query = format!("#{tag}");
        let mut url = tag_url.clone();
        url.query_pairs_mut()
            .clear()
            .append_pair("q", &query)
            .finish();
        url
    };
    let tag_md = |tag: &str| format!("[#{tag}]({})", tag_page(tag));

    match tags.len() {
        0 => String::from("None"),
        1 => tag_md(tags[0].as_ref()),
        2.. => {
            // Always show at least one tag, even if it's too long
            // This could be improved by trying to find the longest tags that fit, but this is fine for now.
            let mut text = tag_md(tags[0].as_ref());
            for (i, tag) in tags[1..].iter().enumerate() {
                let tag_text = tag_md(tag.as_ref());
                // This algorithm isn't perfect because the " and X more" text *may* be truncated, but since it's very likely to be shorter than a single tag (with link), this will be fine.
                let new_length = text.len() + tag_text.len() + 1; // +1 for the space character
                if new_length > 1024 {
                    let remaining = tags.len() - i - 1;
                    if remaining > 0 {
                        write!(text, " and {remaining} more").unwrap();
                    }
                } else {
                    text.push(' ');
                    text.push_str(&tag_text);
                }
            }
            text
        }
    }
}

/// Convert a `bool` value to an English word.
fn human_friendly_bool(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}

/// Create a new Discord client that uses the bot's commands.
#[tracing::instrument(skip(token))]
pub async fn create_client(token: String) -> anyhow::Result<serenity::Client> {
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = Framework::<BotData, anyhow::Error>::builder()
        .options(poise::FrameworkOptions {
            commands: vec![help(), link(), register(), status()],
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
