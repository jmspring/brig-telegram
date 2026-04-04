# brig-telegram

Telegram Bot API gateway for Brig. Bridges Telegram messages to Brig's
unix domain socket.

## What This Is

A standalone, minimal gateway that:
- Long-polls Telegram's getUpdates API
- Forwards message text to Brig via unix socket
- Returns Brig's response to the Telegram chat

Not a bot framework. Not async. Just a bridge.

## Project Structure

```
brig-telegram/
├── Cargo.toml          # 3 deps: ureq, serde, serde_json
├── src/main.rs         # ~280 lines, the entire program
├── manifest.toml       # Brig persistent skill manifest
└── README.md           # Usage instructions
```

## Dependencies

Three crates, no more:
- `ureq` (with json feature) — synchronous HTTP
- `serde` — serialization
- `serde_json` — JSON parsing

No async runtime. No Telegram bot framework.

## Socket Protocol

Newline-delimited JSON over unix domain socket:

```
→ {"type":"hello","name":"telegram-gateway","version":"0.1.0"}
← {"type":"welcome","capabilities":["submit_task","read_status"]}
→ {"type":"task","content":"user message","session":"tg-CHATID-USERID"}
← {"type":"status","skill":"shell","jail":"w-xxx","state":"running"}
← {"type":"response","content":"response text","session":"tg-CHATID-USERID"}
```

Session keys: `tg-{chat_id}-{user_id}`

## Environment Variables

| Variable | Required | Default |
|----------|----------|---------|
| `BRIG_TELEGRAM_TOKEN` | Yes | — |
| `BRIG_SOCKET` | No | `/var/brig/sock/brig.sock` |

## Build & Run

```sh
cargo build --release

# Manual test
BRIG_TELEGRAM_TOKEN=xxx ./target/release/brig-telegram

# Install as brig skill
brig skill add ./
brig secret set telegram-gateway.telegram_token
brig skill enable telegram-gateway
service brig_telegram start
```

## What Works

- Telegram API connection (getUpdates, sendMessage)
- Brig socket handshake (hello/welcome)
- Task submission and response handling
- Reconnection on socket/API errors
- Bot message filtering (ignores messages from bots)

## Design Constraints

From the main Brig project:
- No async — synchronous control flow throughout
- Minimal dependencies — 3 crates only
- Shell out via ureq HTTP, not a Telegram SDK
- Separate repo — gateways don't share code with brig
