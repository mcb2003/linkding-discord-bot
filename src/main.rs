mod linkding_api;
use crate::linkding_api::LinkdingApi;

use anyhow::Context as AnyhowContext;
use poise::serenity_prelude as serenity;

struct Data {
    linkding: LinkdingApi,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN")
        .context("Could not retrieve Discord token from environment")?;
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    linkding: LinkdingApi::from_env()
                        .context("Could not create Linkding client")?,
                })
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .context("Could not create Discord client")?;
    client
        .start()
        .await
        .context("While running Discord client")?;

    Ok(())
}
