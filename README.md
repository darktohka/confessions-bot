# Confessions Bot

A Discord bot written in Rust using the `poise` framework for anonymous confession submissions via threads.

## Features

- Anonymous confession submission.
- Confessions are posted in dedicated threads.
- Supports both slash commands and a confession button.
- Audit logging with size-based rotation (10MB limit).
- Blacklist system to flag confessions containing specific words/phrases for moderator review.
- Moderator review queue for flagged confessions.

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

### 2. Bot Commands

The bot registers the following slash commands:

| Command                         | Description                                                           | Usage                                   |
| :------------------------------ | :-------------------------------------------------------------------- | :-------------------------------------- |
| `/set_confession_thread`        | Sets the channel where new confession threads will be created.        | `/set_confession_thread <channel>`      |
| `/confess`                      | Opens a modal for anonymous confession submission.                    | `/confess`                              |
| `/confessembed`                 | Creates an embed with a button that can open the modal                | `/confessembed`                         |
| `/blacklist add`                | Add a word or phrase to the blacklist (requires Manage Messages)      | `/blacklist add <term>`                 |
| `/blacklist remove`             | Remove a word or phrase from the blacklist (requires Manage Messages) | `/blacklist remove <term>`              |
| `/blacklist list`               | List all blacklisted terms (requires Manage Messages)                 | `/blacklist list`                       |
| `/review_confession list`       | List all pending confessions flagged for review                       | `/review_confession list`               |
| `/review_confession approve`    | Approve and post a flagged confession (requires Manage Messages)      | `/review_confession approve <id>`       |
| `/review_confession reject`     | Reject and remove a flagged confession (requires Manage Messages)     | `/review_confession reject <id>`        |

### 3. Blacklist Feature

Administrators can configure a blacklist of words or phrases that should trigger moderator review before a confession is posted publicly.

**How it works:**
1. Use `/blacklist add` to add terms that should be flagged (case-insensitive matching)
2. When a user submits a confession containing any blacklisted terms, it will be flagged for review instead of being posted immediately
3. The user receives a notification that their confession is pending review
4. Moderators can use `/review_confession list` to see all pending confessions
5. Moderators can approve confessions with `/review_confession approve` to post them, or reject them with `/review_confession reject` to remove them from the queue

**Example workflow:**
```
/blacklist add inappropriate_word
/blacklist add "multi word phrase"
/blacklist list
```

When a confession contains a blacklisted term, the submitter will see:
```
Your confession has been flagged for moderator review because it contains blacklisted terms: inappropriate_word. 
A moderator will review it before it's posted. Confession ID: `abc-123-def`
```

Moderators can then review and approve/reject the confession.

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
