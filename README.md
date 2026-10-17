# Linkding Discord Bot

[![CI](https://github.com/mcb2003/linkding-discord-bot/actions/workflows/ci.yml/badge.svg?branch=main&event=push)](https://github.com/mcb2003/linkding-discord-bot/actions/workflows/ci.yml)
[![CD](https://github.com/mcb2003/linkding-discord-bot/actions/workflows/cd.yml/badge.svg)](https://github.com/mcb2003/linkding-discord-bot/actions/workflows/cd.yml)
[![Latest release](https://img.shields.io/github/v/release/mcb2003/linkding-discord-bot)](https://github.com/mcb2003/linkding-discord-bot/releases/latest)
[![Container image](https://img.shields.io/badge/container-ghcr.io-blue?logo=docker)](https://github.com/mcb2003/linkding-discord-bot/pkgs/container/linkding-discord-bot)

A Discord bot written in Rust using the [poise](https://github.com/serenity-rs/poise) framework (built on top of [serenity](https://github.com/serenity-rs/serenity)). This bot connects to your [Linkding](https://linkding.link) instance via its API, allowing Discord users to manage and share useful links with friends directly from Discord.

> [!WARNING] Early development
> Core features like adding, posting, and retrieving links by ID are functional, but many important features, including access controls, are not yet implemented.

## Features

- **Add new links** to your Linkding instance from Discord
- **Post new and existing links** in Discord channels
- **Retrieve links by ID**
- **Check the reachability of your Linkding instance**
- More features planned!

## Getting Started

### Prerequisites

- Access to a running [Linkding](https://linkding.link) instance and its API token,
- A Discord bot token,
- For source builds, the [Rust toolchain](https://www.rust-lang.org/tools/install) (latest stable).

### Installation

#### Docker

Create a `.env` file as described under [Configuration](#configuration), then run the latest stable image:

```bash
docker run --detach \
  --name linkding-discord-bot \
  --restart unless-stopped \
  --env-file .env \
  ghcr.io/mcb2003/linkding-discord-bot:latest
```

#### Docker Compose

Create a `compose.yaml` alongside your `.env` file:

```yaml
services:
  bot:
    image: ghcr.io/mcb2003/linkding-discord-bot:latest
    restart: unless-stopped
    env_file: .env
```

Start the bot in the background:

```bash
docker compose up --detach
```

#### From Source

Clone the repository

```bash
git clone https://github.com/mcb2003/linkding-discord-bot
cd linkding-discord-bot
```

Compile the project:

```bash
cargo build --release
```

Run it:

```bash
cargo run --release
```

Or:

```bash```
./target/release/linkding-discord-bot
```

### Configuration

The bot is configured with environment variables, either collected from the process environment or read from a `.env` file.

Environment Variable | Purpose
--- | ---
`DISCORD_TOKEN` | Bot token from your Discord application's **Bot** page (get one in the [Developer Portal](https://discord.com/developers/applications)).
`LINKDING_BASE_URL` | Base URL of your Linkding instance, e.g. `https://linkding.example.com`.
`LINKDING_API_TOKEN` | API token used to authenticate requests to your Linkding instance.

## Usage

Once running, invite the bot to your server using a link like this, replacing "XXXXXXXX" with your Discord application's client ID:

    https://discord.com/oauth2/authorize?client_id=XXXXXXXX&permissions=274878154816&integration_type=0&scope=bot+applications.commands

Then use the available commands:

- `/link add <url>` — Add a new link to Linkding
- `/link get <id>` — Retrieve a link by its ID
- `/link post <url> <message>` — Post a link to the current channel

> If you find you can't run any slash commands, you may need to register them in your server. To do so, mention your bot user in a new message with the content "register", then follow the instructions.

More commands and documentation coming soon!

## Roadmap

- More advanced link management features,
- Searching existing links,
- Per server Linkding instances,
- optional per user API tokens,
- Improved error handling and feedback,
- Access controls,
- Automatic tags based on server / channel,
- More ...

## Contributing

Contributions are welcome! Please open [issues](https://github.com/mcb2003/linkding-discord-bot/issues) or [pull requests](https://github.com/mcb2003/linkding-discord-bot/fork).

## License

This project is licensed under the [MIT License](./LICENSE).

---

*This project is not affiliated with Linkding or Discord.*
