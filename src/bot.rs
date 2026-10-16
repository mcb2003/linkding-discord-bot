use std::fmt::Debug;

use anyhow::Context as AnyhowContext;
use poise::{CreateReply, Framework, serenity_prelude as serenity};
use url::Url;

use crate::linkding::{Bookmark, CheckResponse, CreateBookmarkRequest, LinkdingApi};

struct BotData {
    linkding: LinkdingApi,
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
type ApplicationContext<'a> = poise::structs::ApplicationContext<'a, BotData, anyhow::Error>;

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

/// Link management commands. With no subcommand, get a link by its ID.
#[poise::command(
    slash_command,
    prefix_command,
    track_edits,
    subcommands("link_get", "link_add", "link_post")
)]
async fn link(
    ctx: Context<'_>,
    #[description = "The ID of the link to retrieve"] id: u64,
) -> anyhow::Result<()> {
    link_get_internal(ctx, id).await
}

/// Get a link by its ID.
#[poise::command(slash_command, prefix_command, track_edits, rename = "get")]
async fn link_get(
    ctx: Context<'_>,
    #[description = "The ID of the link to retrieve"] id: u64,
) -> anyhow::Result<()> {
    link_get_internal(ctx, id).await
}

#[tracing::instrument(skip(ctx))]
async fn link_get_internal(ctx: Context<'_>, id: u64) -> anyhow::Result<()> {
    let bot = ctx.framework().user_data().await;
    let bookmark = bot.linkding.bookmark(id).await?;

    let embed = bookmark_to_embed(&bookmark, bot.linkding.base_url());
    ctx.send(CreateReply::default().reply(true).embed(embed))
        .await?;

    Ok(())
}

#[derive(Debug, poise::Modal)]
#[name = "Edit Link"]
struct EditLinkModal {
    #[name = "URL"]
    #[placeholder = "https://example.org"]
    url: String,
    #[name = "Tags (separated by space)"]
    tags: Option<String>,
    #[name = "Title"]
    title: String,
    #[name = "Description"]
    #[paragraph]
    description: Option<String>,
    #[name = "Notes"]
    #[paragraph]
    notes: Option<String>,
}

impl From<CheckResponse> for EditLinkModal {
    fn from(response: CheckResponse) -> Self {
        Self {
            url: response.scraped_url().to_string(),
            title: response
                .scraped_title()
                .unwrap_or_else(|| response.scraped_url().as_str())
                .to_string(),
            description: response.scraped_description().map(ToString::to_string),
            notes: None,
            tags: Some(response.auto_tags().join(" ")),
        }
    }
}

impl From<EditLinkModal> for CreateBookmarkRequest {
    fn from(modal: EditLinkModal) -> Self {
        Self {
            url: modal.url,
            title: Some(modal.title),
            description: modal.description,
            notes: modal.notes.unwrap_or_default(),
            tag_names: modal
                .tags
                .map(|s| s.split(' ').map(ToString::to_string).collect())
                .unwrap_or_default(),
            is_archived: false,
            unread: false,
            shared: true,
        }
    }
}

/// Add a new link.
#[poise::command(slash_command, track_edits, rename = "add")]
async fn link_add(
    ctx: ApplicationContext<'_>,
    #[description = "The link to add"] link: String,
) -> anyhow::Result<()> {
    link_add_internal(ctx, link, true, "✅ Link created!", false).await
}

/// Add a new link, and post it to the channel with a message.
#[poise::command(slash_command, track_edits, rename = "post")]
async fn link_post(
    ctx: ApplicationContext<'_>,
    #[description = "The link to add"] link: String,
    #[description = "The content of the message"]
    #[rest]
    message_content: String,
) -> anyhow::Result<()> {
    link_add_internal(ctx, link, false, message_content, true).await
}

#[tracing::instrument(skip(ctx))]
async fn link_add_internal(
    ctx: ApplicationContext<'_>,
    link: String,
    ephemeral_response: bool,
    response_content: impl Into<String> + Debug,
    allow_repost: bool,
) -> anyhow::Result<()> {
    let bot = ctx.framework().user_data().await;
    let url = link.parse().context("Invalid URL")?;

    let response = bot
        .linkding
        .check_bookmark(url)
        .await
        .context("Could not communicate with the Linkding API")?;

    if let Some(bookmark) = response.bookmark() {
        let embed = bookmark_to_embed(bookmark, bot.linkding.base_url());
        let reply = if allow_repost {
            // Skip editting, just post immediately
            CreateReply::default()
                .reply(true)
                .ephemeral(ephemeral_response)
                .content(response_content)
                .embed(embed)
        } else {
            // Warn the user that the bookmark is already saved
            CreateReply::default()
                .reply(true)
                .ephemeral(true)
                .content("⚠️ Link already exists")
                .embed(embed)
        };
        ctx.send(reply).await?;
        return Ok(());
    }

    let modal: EditLinkModal = response.into();
    let req: CreateBookmarkRequest =
        match poise::modal::execute_modal(ctx, Some(modal), None).await? {
            Some(r) => r.into(),
            None => return Ok(()), // Cancelled
        };
    let bookmark = bot.linkding.create_bookmark(req).await?;
    let reply = CreateReply::default()
        .reply(true)
        .ephemeral(ephemeral_response)
        .content(response_content)
        .embed(bookmark_to_embed(&bookmark, bot.linkding.base_url()));
    ctx.send(reply).await?;

    Ok(())
}

/// Convert a [`Bookmark`] into a Discord embed.
fn bookmark_to_embed(bookmark: &Bookmark, linkding_base_url: &Url) -> serenity::CreateEmbed {
    let mut embed = serenity::CreateEmbed::default()
        .title(&bookmark.title)
        .description(&bookmark.description)
        .url(&bookmark.url)
        .timestamp(bookmark.date_added);

    let tags = tags_to_md(&bookmark.tag_names, linkding_base_url);
    embed = embed.field("Tags", tags, false);

    if let Some(archive_url) = &bookmark.web_archive_snapshot_url {
        let value = format!("[view on the Wayback Machine]({})", archive_url);
        embed = embed.field("Snapshot", value, true);
    }

    if !bookmark.notes.is_empty() {
        embed = embed.field("Notes", &bookmark.notes, false);
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

    if let Some(image) = &bookmark.preview_image_url {
        embed = embed.image(image);
    }
    if let Some(icon) = &bookmark.favicon_url {
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

/// Create a new Discord client that uses the bot's commands.
#[tracing::instrument(skip(token))]
pub async fn create_client(token: String) -> anyhow::Result<serenity::Client> {
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = Framework::<BotData, anyhow::Error>::builder()
        .options(poise::FrameworkOptions {
            commands: vec![link(), register(), status()],
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
