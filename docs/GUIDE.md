# brig-telegram: Installation, Configuration, and Usage Guide

## Overview

brig-telegram is a gateway that bridges Telegram messages to Brig. When a
user sends a message to the Telegram bot, brig-telegram forwards it to
Brig's unix domain socket, waits for the LLM-driven response, and sends
the result back to the Telegram chat.

No async runtime, no Telegram bot framework. Synchronous HTTP via ureq
and blocking socket I/O in ~350 lines.

## Prerequisites

- FreeBSD with Brig installed and running in daemon mode (`brig -d`)
- A working internet connection (the gateway needs to reach api.telegram.org)
- Rust toolchain (for building from source)

## Step 1: Register a Telegram Bot

You need a bot account on Telegram before brig-telegram can connect. All
Telegram bot registration goes through BotFather, Telegram's official bot
management interface.

### Create the Bot

1. Open Telegram and search for **@BotFather**, or go to https://t.me/BotFather
2. Start a conversation with BotFather and send:

   ```
   /newbot
   ```

3. BotFather asks for a **display name** for the bot. This is what users see
   in chat (e.g., "Brig Assistant"). It can contain spaces and doesn't need
   to be unique.

4. BotFather asks for a **username**. This must:
   - End in `bot` (e.g., `brig_assistant_bot` or `MyBrigBot`)
   - Be unique across all of Telegram
   - Contain only alphanumeric characters and underscores

5. BotFather responds with your **bot token**, which looks like:

   ```
   7123456789:AAHfKz3xJ9Wq_mN5vRtY8pL2dBcEaGhIjKl
   ```

   **Copy this token immediately.** You can retrieve it later with
   `/token` in BotFather, but treat it as a secret.

### Configure Bot Settings (Optional)

These settings are all configured through BotFather commands:

**Set a description** (shown when users first open a chat with the bot):

```
/setdescription
```

Select your bot, then enter a description like:
"FreeBSD system assistant powered by Brig. Send me a task in natural language."

**Set an about text** (shown in the bot's profile):

```
/setabouttext
```

**Set a profile photo**:

```
/setuserpic
```

**Disable group join** (if you want the bot to only work in direct messages):

```
/setjoingroups
```

Select your bot, then choose "Disable".

**Set bot commands** (shows a menu of commands in the Telegram UI):

```
/setcommands
```

This is optional. brig-telegram treats all message text equally -- it doesn't
use Telegram's command system. But you could set a help command for user
convenience:

```
help - Show what this bot can do
```

### Privacy Mode

By default, bots in group chats only receive messages that:
- Start with `/` (commands)
- Are replies to the bot's messages
- Mention the bot by @username

To let the bot see all messages in a group (not just commands), disable
privacy mode:

```
/setprivacy
```

Select your bot, then choose "Disable".

For direct-message-only usage, privacy mode doesn't matter.

## Step 2: Build brig-telegram

```sh
cd /path/to/brig-telegram
cargo build --release
```

The binary is at `target/release/brig-telegram`.

## Step 3: Install

### Option A: Install as a Brig Persistent Skill (Recommended)

This runs brig-telegram inside a FreeBSD jail with network access restricted
to api.telegram.org.

```sh
# Install the binary where the jail can reach it
sudo cp target/release/brig-telegram /usr/local/bin/

# Register the skill manifest with brig
brig skill add /path/to/brig-telegram/

# Store the bot token (encrypted in ~/.brig/secrets.db)
brig secret set telegram-gateway.telegram_token
# Paste your bot token when prompted

# Enable the skill (creates ZFS dataset, jail, rc.d script)
sudo brig skill enable telegram-gateway

# Start the service
sudo sysrc brig_telegram_enable=YES
sudo service brig_telegram start
```

Check that it's running:

```sh
sudo service brig_telegram status
```

View logs via syslog:

```sh
grep brig_telegram /var/log/messages
```

### Option B: Run Manually (For Testing)

```sh
export BRIG_TELEGRAM_TOKEN="your-bot-token"
./target/release/brig-telegram
```

The gateway prints status to stderr:

```
brig-telegram starting
  socket: /var/brig/sock/brig.sock
connected to brig, capabilities: ["submit_task", "read_status"]
polling for updates...
```

With a custom socket path:

```sh
BRIG_TELEGRAM_TOKEN="your-bot-token" \
BRIG_SOCKET="/path/to/brig.sock" \
./target/release/brig-telegram
```

## Step 4: Configure

### Environment Variables

| Variable              | Required | Default                       | Description                    |
|-----------------------|----------|-------------------------------|--------------------------------|
| `BRIG_TELEGRAM_TOKEN` | Yes      | --                            | Bot token from @BotFather      |
| `BRIG_SOCKET`         | No       | `/var/brig/sock/brig.sock`    | Path to Brig's unix socket     |

When installed as a persistent skill, `BRIG_TELEGRAM_TOKEN` is injected
automatically from brig's secret store. Override `BRIG_SOCKET` only if
your brig daemon uses a non-default socket path.

### Brig Daemon Configuration

Ensure brig is running in daemon mode and the socket is accessible:

```sh
# Start brig daemon
brig -d

# Verify the socket exists
ls -la /var/brig/sock/brig.sock
```

The gateway connects with `submit_task` and `read_status` capabilities.
No additional brig configuration is needed.

### Skill Manifest

The `manifest.toml` declares what the gateway needs:

```toml
[skill]
name = "telegram-gateway"
description = "Bridge Telegram messages to Brig"
kind = "persistent"

[requires]
network = ["api.telegram.org"]
max_runtime = "forever"

[persistent]
rc_name = "brig_telegram"
entrypoint = "/usr/local/bin/brig-telegram"
restart_on_failure = true
depends_on = ["brig"]

[persistent.socket]
capabilities = ["submit_task", "read_status"]

[secrets]
telegram_token = { env = "BRIG_TELEGRAM_TOKEN" }
```

You shouldn't need to modify this unless your installation paths differ.

### Polling Behavior

brig-telegram uses Telegram's long-polling API (`getUpdates`) with a
30-second timeout. This means:

- The bot checks for new messages every 30 seconds at most
- When a message arrives during polling, it's delivered immediately
- No webhook server or public IP address is required
- The polling timeout adds 5 seconds for the HTTP read timeout (35s total)

## Usage

### Basic Interaction

Once the bot is running, open a chat with it in Telegram and send a message:

```
You:  What's the disk usage on the system?
Bot:  All ZFS pools healthy. zroot is at 42% capacity (126G used of 300G).
```

```
You:  List the installed packages in the base template
Bot:  Base template has 47 packages installed:
      bash-5.2.21, ca_root_nss-3.93, curl-8.5.0, ...
```

```
You:  Remember that the next maintenance window is April 20th
Bot:  Noted. I'll remember that the next maintenance window is 2026-04-20.
```

### Group Chat Usage

brig-telegram works in group chats. Add the bot to a group and:

- If **privacy mode is disabled** (via BotFather), the bot sees all messages
  and will respond to each one
- If **privacy mode is enabled** (default), the bot only sees:
  - Messages starting with `/`
  - Replies to the bot's own messages
  - Messages that @mention the bot

For most uses, you'll want the bot in a dedicated channel or in direct
messages to avoid responding to every message in a busy group.

### Session Isolation

Each conversation is tracked by a session key:

```
tg-{chat_id}-{user_id}
```

This means:

- **Different users in the same group** have separate sessions and
  separate memory contexts
- **The same user in different chats** has separate sessions
- **Direct messages** use the DM chat_id, which is unique per user

Brig's memory system associates facts and context with these session keys,
so conversations maintain continuity across messages.

### Error Handling

If Brig is unavailable or returns an error, the bot sends the error message
to the Telegram chat:

```
Bot:  Error: brig error task_failed: skill shell timed out after 120s
```

If the Brig socket disconnects, the gateway attempts to reconnect
automatically. If the reconnection succeeds, it retries the failed message.
If it fails, the error is reported to the user.

Telegram API errors (e.g., rate limiting) are logged to stderr but don't
crash the gateway. The polling loop continues.

## Troubleshooting

### Bot doesn't respond to messages

- Check that the gateway process is running: `service brig_telegram status`
- Check logs: `grep brig_telegram /var/log/messages`
- Verify the token is correct: send `/token` to @BotFather and compare with
  what's stored in `brig secret list`
- In group chats, check privacy mode settings (see Step 1)

### "BRIG_TELEGRAM_TOKEN environment variable not set"

- When running manually, export the variable first
- When running as a persistent skill, set it via `brig secret set`

### "failed to connect to brig socket"

- Verify brig is running in daemon mode: `brig status`
- Check the socket path: `ls -la /var/brig/sock/brig.sock`
- If using a custom socket path, set `BRIG_SOCKET` accordingly

### "brig does not grant submit_task capability"

- The brig daemon may not be configured to accept gateway connections
- Check that the skill manifest is registered: `brig skill info telegram-gateway`

### "Telegram API error: Unauthorized"

- The bot token is invalid or has been revoked
- Generate a new token via `/token` or `/revoke` in @BotFather
- Update the stored secret: `brig secret set telegram-gateway.telegram_token`

### "getUpdates failed: Conflict"

- Another instance of the bot is running with the same token. Telegram only
  allows one long-polling connection per bot token at a time.
- Stop the other instance, or check for duplicate services.

## Stopping the Service

```sh
sudo service brig_telegram stop
```

To disable at boot:

```sh
sudo sysrc brig_telegram_enable=NO
```

To remove the skill entirely:

```sh
sudo service brig_telegram stop
sudo brig skill disable telegram-gateway
brig skill remove telegram-gateway
sudo rm /usr/local/bin/brig-telegram
```

## Running Multiple Bots

You can run multiple instances of brig-telegram on the same host, each with
its own Telegram bot, its own skill registration, and its own session
prefix. This is useful when you want separate bots for different teams or
purposes (e.g., an ops bot and a support bot).

### Register Each Bot with @BotFather

Create a separate Telegram bot for each instance. Follow the same process
described in Step 1 above, once per bot. You'll end up with a separate
bot token for each.

For example:
- `ops_brig_bot` with token `7111111111:AAH...` for the ops team
- `support_brig_bot` with token `7222222222:AAH...` for the support team

### Create a Manifest per Bot

The `contrib/` directory includes example manifests:

- `contrib/manifest-ops.toml` -- ops team bot
- `contrib/manifest-support.toml` -- support team bot

Each manifest sets a unique skill name, rc.d service name, and session
prefix via the `env` table in the `[persistent]` section:

```toml
[persistent]
rc_name = "brig_tg_ops"
entrypoint = "/usr/local/bin/brig-telegram"
restart_on_failure = true
depends_on = ["brig"]
env = { BRIG_GATEWAY_NAME = "telegram-ops-bot", BRIG_SESSION_PREFIX = "tg-ops" }
```

The `BRIG_SESSION_PREFIX` controls the session key format
(`{prefix}-{chat_id}-{user_id}`), so each bot's conversations are isolated
in Brig's session store.

Copy and edit a manifest from `contrib/` if you need different settings.

### Install the Binary Once

All instances share the same binary. Build and install it once:

```sh
cargo build --release
sudo cp target/release/brig-telegram /usr/local/bin/
```

### Register Each Bot as a Separate Skill

```sh
brig skill add --manifest contrib/manifest-ops.toml
brig skill add --manifest contrib/manifest-support.toml
```

### Set Secrets for Each

Each skill has its own secret namespace, keyed by skill name:

```sh
brig secret set telegram-ops-bot.telegram_token
# Paste the ops bot token when prompted

brig secret set telegram-support-bot.telegram_token
# Paste the support bot token when prompted
```

### Enable and Start Each

```sh
sudo brig skill enable telegram-ops-bot
sudo sysrc brig_tg_ops_enable=YES
sudo service brig_tg_ops start

sudo brig skill enable telegram-support-bot
sudo sysrc brig_tg_support_enable=YES
sudo service brig_tg_support start
```

Verify both are running:

```sh
sudo service brig_tg_ops status
sudo service brig_tg_support status
```

### Session and Memory Isolation

Each bot uses a different session prefix (`tg-ops`, `tg-support`, etc.),
so their session keys never collide. A message to the ops bot produces
session key `tg-ops-12345-67890`, while the same user messaging the
support bot gets `tg-support-12345-67890`.

Memory facts in Brig are scoped per user, not per gateway. If a user tells
the ops bot "remember that the maintenance window is Friday" and then asks
the support bot "when is the maintenance window?", the support bot can
recall that fact because memory is associated with the user's identity
across gateways. Session history, however, is separate -- each bot
maintains its own conversation thread.

## Revoking the Bot

If you need to decommission the bot entirely:

1. Stop the gateway service
2. Send `/deletebot` to @BotFather in Telegram
3. Select your bot and confirm deletion
4. Remove the secret: `brig secret remove telegram-gateway.telegram_token`
5. Remove the skill: `brig skill remove telegram-gateway`
