mod bot;
mod linkding_api;

use anyhow::Context as AnyhowContext;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN")
        .context("Could not retrieve Discord token from environment")?;

    let mut client = bot::create_client(token)
        .await
        .context("Could not create Discord client")?;

    client
        .start()
        .await
        .context("While running Discord client")?;

    Ok(())
}
