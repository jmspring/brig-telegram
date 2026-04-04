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

## Install as a Brig persistent skill

```sh
# 1. Build release binary
cargo build --release

# 2. Install binary to system path
sudo cp target/release/brig-telegram /usr/local/bin/

# 3. Register skill with brig (loads manifest into registry)
brig skill add ./

# 4. Set the Telegram token (stored encrypted in ~/.brig/secrets.db)
brig secret set telegram-gateway.telegram_token
# (enter your bot token when prompted)

# 5. Enable the skill (creates ZFS dataset, jail.conf, rc.d script)
sudo brig skill enable telegram-gateway

# 6. Start the service
sudo sysrc brig_telegram_enable=YES
sudo service brig_telegram start
```

Note: Steps 5-6 require root. The gateway runs inside a FreeBSD jail
with network access limited to api.telegram.org.

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
   - Creates a session key: `tg-{chat_id}-{user_id}`
   - Submits the message text to Brig
   - Waits for Brig's response
   - Sends the response back to the Telegram chat
5. Handles reconnection on socket or API errors

## Socket protocol

The gateway uses Brig's newline-delimited JSON protocol:

```
→ {"type":"hello","name":"telegram-gateway","version":"0.1.0"}
← {"type":"welcome","capabilities":["submit_task","read_status"]}
→ {"type":"task","content":"user message","session":"tg-12345-67890"}
← {"type":"status","skill":"shell","jail":"w-xxx","state":"running"}
← {"type":"response","content":"response text","session":"tg-12345-67890"}
```

## Environment variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `BRIG_TELEGRAM_TOKEN` | Yes | - | Telegram bot token from @BotFather |
| `BRIG_SOCKET` | No | `/var/brig/sock/brig.sock` | Path to Brig's unix socket |

## License

BSD-2-Clause
