# Confessions Bot

A Discord bot written in Rust using the `poise` framework for anonymous confession submissions via threads.

## Features

- Anonymous confession submission.
- Confessions are posted in dedicated threads.
- Supports both slash commands and a confession button.
- Audit logging with size-based rotation (10MB limit).

## Setup and Configuration

The bot requires a `config.json` file in the root directory to run.

### 1. Create `config.json`

When the bot runs for the first time, it will create a default `config.json` file and exit. You must edit this file with your Discord Bot Token.

```json
{
  "discord_token": "YOUR_BOT_TOKEN_HERE",
  "confession_threads": {}
}
```

- **`discord_token`**: Replace `"YOUR_BOT_TOKEN_HERE"` with your actual Discord bot token.
- **`confession_threads`**: This map is automatically managed by the bot and stores which channel ID is designated for new confession threads in each guild (server).

The bot also creates and manages a `button_stats.toml` file that tracks how many times each user has pressed the confession button. This file is automatically created and updated by the bot.

### 2. Bot Commands

The bot registers the following slash commands:

| Command                  | Description                                                    | Usage                              |
| :----------------------- | :------------------------------------------------------------- | :--------------------------------- |
| `/set_confession_thread` | Sets the channel where new confession threads will be created. | `/set_confession_thread <channel>` |
| `/confess`               | Opens a modal for anonymous confession submission.             | `/confess`                         |
| `/confessembed`          | Creates an embed with a button that can open the modal         | `/confessembed`                    |
| `/buttonstats`           | Shows how many times each user has pressed the confession button. | `/buttonstats`                     |

## Running the Bot

You can run the bot either by building it from source or by downloading a pre-built artifact from GitHub Actions.

### Option A: Build from Source (Recommended)

This requires the Rust toolchain to be installed.

1.  **Clone the repository:**

    ```bash
    git clone https://github.com/darktohka/confessions-bot
    cd confessions-bot
    ```

2.  **Build the binary:**
    The project uses the `release-lto` profile for optimized, stripped binaries.

    ```bash
    cargo build --release --profile release-lto
    ```

3.  **Run the bot:**
    The executable will be located at `target/release-lto/confessions-bot`.
    ```bash
    ./target/release-lto/confessions-bot
    ```

### Option B: Download GitHub Actions Artifact

Pre-built binaries are available for Linux targets via GitHub Actions artifacts.

1.  Go to the [Actions tab](https://github.com/darktohka/confessions-bot/actions) of the repository.
2.  Select the latest successful workflow run (e.g., "Build binaries").
3.  Scroll down to the **Artifacts** section.
4.  Download the appropriate artifact for your system:
    - `binary-x86_64`: For standard 64-bit Linux systems.
    - `binary-aarch64`: For ARM64 Linux systems (e.g., Raspberry Pi 4).
5.  Extract the downloaded archive. It will contain the `confessions-bot` executable.
6.  Make the binary executable and run it:
    ```bash
    chmod +x confessions-bot
    ./confessions-bot
    ```
