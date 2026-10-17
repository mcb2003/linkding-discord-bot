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
    if tags.is_empty() {
        return "None".into();
    }

    // Materialise markdown links
    let links = tags
        .iter()
        .map(|tag| {
            let hashtag = format!("#{}", tag.as_ref());
            let mut url = base_url.join("bookmarks").expect("Bookmarks path is valid");
            url.query_pairs_mut().append_pair("q", &hashtag);
            format!("[{hashtag}]({url})")
        })
        .collect::<Vec<_>>();

    let decimal_digits = |i: usize| if i == 0 { 1 } else { i.ilog10() as usize + 1 };
    let mut char_count = 0;
    let mut best_index = None;
    for (i, link) in links.iter().enumerate() {
        let new_char_count = char_count + link.chars().count() + if i != 0 { 1 } else { 0 };

        let omitted = links.len() - i - 1;
        let trailor_len = if omitted > 0 {
            10 + decimal_digits(omitted) // 10 == " and  more".len()
        } else {
            0 // No trailing text
        };

        if new_char_count + trailor_len > 1024 {
            break; // Too long
        }

        char_count = new_char_count;
        best_index = Some(i);
    }

    match best_index {
        None => tags.len().to_string(),
        Some(i) => {
            use std::fmt::Write;

            let mut text = links[..=i].join(" ");
            let remaining = tags.len() - i - 1;
            if remaining != 0 {
                write!(&mut text, " and {remaining} more").unwrap();
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

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn check_response(metadata: Value, auto_tags: Value) -> CheckResponse {
        serde_json::from_value(json!({
            "bookmark": null,
            "metadata": metadata,
            "auto_tags": auto_tags,
        }))
        .expect("fixture should be a valid check response")
    }

    fn bookmark() -> Bookmark {
        serde_json::from_value(json!({
            "id": 42,
            "url": "https://example.org/article",
            "title": "Example Article",
            "description": "A description",
            "notes": "Read this before Friday",
            "web_archive_snapshot_url": "https://web.archive.org/example",
            "favicon_url": "https://example.org/favicon.ico",
            "preview_image_url": "https://example.org/preview.png",
            "is_archived": true,
            "unread": true,
            "shared": true,
            "tag_names": ["example", "article"],
            "date_added": "2026-10-17T12:34:56Z",
            "date_modified": "2026-10-17T13:00:00Z",
            "website_title": "Example",
            "website_description": "Example website",
        }))
        .expect("fixture should be a valid bookmark")
    }

    #[test]
    fn checked_page_metadata_populates_edit_modal() {
        let response = check_response(
            json!({
                "url": "https://example.org/article",
                "title": "Example Article",
                "description": "A description",
            }),
            json!(["example", "article"]),
        );

        let modal: EditLinkModal = response.into();

        assert_eq!(modal.url, "https://example.org/article");
        assert_eq!(modal.title, "Example Article");
        assert_eq!(modal.description.as_deref(), Some("A description"));
        assert_eq!(modal.notes, None);
        assert_eq!(modal.tags.as_deref(), Some("example article"));
    }

    #[test]
    fn checked_page_without_title_uses_url_as_title() {
        let response = check_response(
            json!({
                "url": "https://example.org/untitled",
                "title": null,
                "description": null,
            }),
            json!([]),
        );

        let modal: EditLinkModal = response.into();

        assert_eq!(modal.title, "https://example.org/untitled");
        assert_eq!(modal.description, None);
    }

    #[test]
    fn edit_modal_populates_create_request_and_normalises_tags() {
        let modal = EditLinkModal {
            url: "https://example.org/article".into(),
            // Whitespace should be collapsed
            tags: Some(" example   article\nfriday ".into()),
            title: "Example Article".into(),
            description: Some("A description".into()),
            notes: Some("Read this before Friday".into()),
        };

        let request: CreateBookmarkRequest = modal.into();

        assert_eq!(request.url, "https://example.org/article");
        assert_eq!(request.title.as_deref(), Some("Example Article"));
        assert_eq!(request.description.as_deref(), Some("A description"));
        assert_eq!(request.notes, "Read this before Friday");
        assert_eq!(request.tag_names, ["example", "article", "friday"]);
        assert!(!request.is_archived);
        assert!(!request.unread);
        assert!(request.shared);
    }

    #[test]
    fn missing_optional_modal_values_become_empty_request_values() {
        let modal = EditLinkModal {
            url: "https://example.org/article".into(),
            tags: None,
            title: "Example Article".into(),
            description: None,
            notes: None,
        };

        let request: CreateBookmarkRequest = modal.into();

        assert_eq!(request.description, None);
        assert!(request.notes.is_empty());
        assert!(request.tag_names.is_empty());
    }

    #[test]
    fn tag_markdown_handles_empty_and_url_encoded_tags() {
        let base_url = Url::parse("https://links.example/").unwrap();
        let no_tags: [&str; 0] = [];

        assert_eq!(tags_to_md(&no_tags, &base_url), "None");
        assert_eq!(
            tags_to_md(&["article"], &base_url),
            "[#article](https://links.example/bookmarks?q=%23article)"
        );
    }

    #[test]
    fn tag_markdown_stays_within_discord_field_limit() {
        let base_url = Url::parse("https://links.example/").unwrap();
        let tags = (0..30)
            .map(|index| format!("tag-{index}-{}", "x".repeat(80)))
            .collect::<Vec<_>>();

        let markdown = tags_to_md(&tags, &base_url);

        assert!(markdown.chars().count() <= 1024);
        assert!(markdown.ends_with(" more"));
        assert!(!markdown.contains("tag-29-"));
    }

    #[test]
    fn bookmark_populates_discord_embed() {
        let bookmark = bookmark();
        let base_url = Url::parse("https://links.example/").unwrap();

        let embed = serde_json::to_value(bookmark_to_embed(&bookmark, &base_url))
            .expect("embed should be serializable");

        assert_eq!(embed["title"], "Example Article");
        assert_eq!(embed["description"], "A description");
        assert_eq!(embed["url"], "https://example.org/article");
        assert_eq!(
            embed["fields"],
            json!([
                {
                    "name": "Tags",
                    "value": "[#example](https://links.example/bookmarks?q=%23example) [#article](https://links.example/bookmarks?q=%23article)",
                    "inline": false
                },
                {
                    "name": "Snapshot",
                    "value": "[view on the Wayback Machine](https://web.archive.org/example)",
                    "inline": true
                },
                {
                    "name": "Notes",
                    "value": "Read this before Friday",
                    "inline": false
                }
            ])
        );
        assert_eq!(
            embed["footer"]["text"],
            "📥 Unread | 🔗 Shared | 🗃️ Archived"
        );
        assert_eq!(embed["image"]["url"], "https://example.org/preview.png");
        assert_eq!(embed["thumbnail"]["url"], "https://example.org/favicon.ico");
        assert!(embed["timestamp"].is_string());
    }

    #[test]
    fn bookmark_without_optional_data_produces_minimal_embed() {
        let mut bookmark = bookmark();
        bookmark.notes.clear();
        bookmark.web_archive_snapshot_url = None;
        bookmark.favicon_url = None;
        bookmark.preview_image_url = None;
        bookmark.is_archived = false;
        bookmark.unread = false;
        bookmark.shared = false;
        bookmark.tag_names.clear();

        let embed = serde_json::to_value(bookmark_to_embed(
            &bookmark,
            &Url::parse("https://links.example/").unwrap(),
        ))
        .expect("embed should be serializable");

        assert_eq!(
            embed["fields"],
            json!([{"name": "Tags", "value": "None", "inline": false}])
        );
        assert!(embed.get("footer").is_none());
        assert!(embed.get("image").is_none());
        assert!(embed.get("thumbnail").is_none());
    }
}
