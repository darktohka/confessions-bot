# Confessions Bot

A Discord bot written in Rust using the `poise` framework for anonymous confession submissions via threads.

## Features

- Anonymous confession submission.
- Confessions are posted in dedicated threads.
- Supports both slash commands and a confession button.
- Configurable cooldown periods to prevent spam (default: 1 hour).
- Audit logging with size-based rotation (10MB limit).

## Setup and Configuration

The bot requires a `config.json` file in the root directory to run.

### 1. Create `config.json`

When the bot runs for the first time, it will create a default `config.json` file and exit. You must edit this file with your Discord Bot Token.

```json
{
  "discord_token": "YOUR_BOT_TOKEN_HERE",
  "confession_threads": {},
  "cooldowns": {}
}
```

- **`discord_token`**: Replace `"YOUR_BOT_TOKEN_HERE"` with your actual Discord bot token.
- **`confession_threads`**: This map is automatically managed by the bot and stores which channel ID is designated for new confession threads in each guild (server).
- **`cooldowns`**: Optional map of guild IDs to cooldown periods (in seconds). Defaults to 3600 seconds (1 hour) if not specified. Set to 0 to disable cooldowns for a guild.

### 2. Bot Commands

The bot registers the following slash commands:

| Command                  | Description                                                    | Usage                              |
| :----------------------- | :------------------------------------------------------------- | :--------------------------------- |
| `/set_confession_thread` | Sets the channel where new confession threads will be created. | `/set_confession_thread <channel>` |
| `/set_cooldown`          | Sets the cooldown period between confession submissions.       | `/set_cooldown <seconds>`          |
| `/confess`               | Opens a modal for anonymous confession submission.             | `/confess`                         |
| `/confessembed`          | Creates an embed with a button that can open the modal         | `/confessembed`                    |

### 3. Cooldown System

The bot includes a configurable cooldown system to prevent spam and ensure fair usage:

- **Default Cooldown**: 1 hour (3600 seconds) between confessions per user
- **Guild-Specific**: Each server can set its own cooldown period using `/set_cooldown`
- **Disable Cooldown**: Set cooldown to 0 seconds to allow unlimited confessions
- **Anonymous Enforcement**: Cooldowns are tracked by user ID but maintain anonymity for public confessions
- **Cooldown Messages**: Users receive ephemeral messages showing how much time remains

**Example Usage:**
```
/set_cooldown 1800    # Set cooldown to 30 minutes
/set_cooldown 7200    # Set cooldown to 2 hours  
/set_cooldown 0       # Disable cooldown completely
```

When a user tries to confess before their cooldown expires, they receive a private message like:
> "You must wait 45 minutes and 30 seconds before submitting another confession."

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
