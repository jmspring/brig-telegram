# brig-telegram

Telegram Bot API gateway for [Brig](https://github.com/jmspring/brig).

A minimal, synchronous bridge that forwards Telegram messages to Brig's unix domain socket and returns responses. No async, no bot framework — just ureq HTTP calls and blocking socket I/O.

## Prerequisites

- Brig running in daemon mode (`brig -d`)
- A Telegram bot token from [@BotFather](https://t.me/BotFather)

## Build

```sh
cargo build --release
```

## Install

```sh
make                     # build release binary
sudo make install        # install binary + skill manifest
```

This installs:
- `/usr/local/bin/brig-telegram`
- `/usr/local/share/brig/skills/telegram-gateway/manifest.toml`

Then enable via brig (jailed, recommended):

```sh
brig secret set telegram-gateway.telegram_token
brig skill enable telegram-gateway
```

Or as a host service (no jail):

```sh
sudo make install-service
sudo sysrc brig_telegram_enable=YES
sudo sysrc brig_telegram_token="your-bot-token"
sudo sysrc brig_telegram_user="jim"
sudo service brig_telegram start
```

## Manual run (for testing)

```sh
BRIG_TELEGRAM_TOKEN=your_bot_token ./target/release/brig-telegram
```

Or with a custom socket path:

```sh
BRIG_TELEGRAM_TOKEN=your_bot_token \
BRIG_SOCKET=/path/to/brig.sock \
./target/release/brig-telegram
```

## How it works

1. Connects to Brig's unix socket at `/var/brig/sock/brig.sock` (or `BRIG_SOCKET`)
2. Sends a hello handshake, receives capabilities
3. Long-polls Telegram's getUpdates API (30s timeout)
4. For each incoming message:
   - Creates a session key: `{session_prefix}-{chat_id}-{user_id}` (default prefix: `tg`)
   - Submits the message text to Brig
   - Waits for Brig's response
   - Sends the response back to the Telegram chat
5. Handles reconnection on socket or API errors

## Socket protocol

The gateway uses Brig's newline-delimited JSON protocol:

```
→ {"type":"hello","name":"telegram-gateway","version":"0.4.0"}
← {"type":"welcome","capabilities":["submit_task","read_status"]}
→ {"type":"task","content":"user message","session":"tg-12345-67890"}
← {"type":"status","skill":"shell","jail":"w-xxx","state":"running"}
← {"type":"response","content":"response text","session":"tg-12345-67890"}
```

## Environment variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `BRIG_TELEGRAM_TOKEN` | Yes | - | Telegram bot token from @BotFather |
| `BRIG_TOKEN` | Yes | - | Brig IPC authentication token (generate with `brig token create telegram-gateway`) |
| `BRIG_SOCKET` | No | `~/.brig/sock/brig.sock` | Path to Brig's unix socket |
| `BRIG_GATEWAY_NAME` | No | `telegram-gateway` | Gateway identity for brig (audit/logging) |
| `BRIG_SESSION_PREFIX` | No | `tg` | Session key prefix (e.g., `tg-{chat_id}-{user_id}`) |
| `BRIG_TELEGRAM_ALLOWED_USERS` | No | all users | Comma-separated Telegram user IDs to accept messages from |

## Running Multiple Bots

You can run multiple instances of brig-telegram on the same host, each as a separate Brig persistent skill with its own bot token and session prefix. See [docs/GUIDE.md](docs/GUIDE.md#running-multiple-bots) for setup instructions. Example manifests are in `contrib/`.

## License

BSD-2-Clause
