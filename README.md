# Linkding Discord Bot

[![CI](https://github.com/mcb2003/linkding-discord-bot/actions/workflows/ci.yml/badge.svg?branch=main&event=push)](https://github.com/mcb2003/linkding-discord-bot/actions/workflows/ci.yml)

A Discord bot written in Rust using the [poise](https://github.com/serenity-rs/poise) framework (built on top of [serenity](https://github.com/serenity-rs/serenity)). This bot connects to your [Linkding](https://linkding.link) instance via its API, allowing Discord users to manage and share useful links with friends directly from Discord.

> **Status:** Early development. Core features like adding, posting, and retrieving links by ID are functional, but many important features (such as access controls) are not yet implemented.

## Features

- **Add new links** to your Linkding instance from Discord
- **Post new and existing links** in Discord channels
- **Retrieve links by ID**
- **Check the reachability of your Linkding instance**
- More features planned!

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- Access to a running [Linkding](https://linkding.link) instance and its API token
- Discord bot token

### Installation

Currently, the only way to run the bot is to compile from source.

```bash
git clone https://github.com/mcb2003/linkding-discord-bot
cd linkding-discord-bot
cargo build --release
```

### Configuration

Create a `.env` file in the project root with the following variables:

```env
DISCORD_TOKEN=your_discord_bot_token
LINKDING_BASE_URL=https://your-linkding-instance.com
LINKDING_API_TOKEN=your_linkding_api_token
```

### Running

```bash
cargo run --release
```

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

- Container images (Docker) for easy deployment
- More advanced link management features
- Improved error handling and feedback
- Access controls
- Automatic tags based on server / channel
- More ...

## Contributing

Contributions are welcome! Please open [issues](https://github.com/mcb2003/linkding-discord-bot/issues) or [pull requests](https://github.com/mcb2003/linkding-discord-bot/fork).

## License

This project is licensed under the [MIT License](./LICENSE).

---

*This project is not affiliated with Linkding or Discord.*
