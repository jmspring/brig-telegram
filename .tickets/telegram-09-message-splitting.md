---
id: telegram-09
status: done
wave: 12
version: v0.2.0
priority: 2
type: bug
tags: [reliability, telegram-api]
deps: []
---
# Add message splitting for long responses

## Problem

Telegram's sendMessage API has a 4096-character limit. When a brig response exceeds this, the API returns an error and the user receives no response at all. The Discord gateway already has a `split_message()` function for its 2000-char limit, but the Telegram gateway has no equivalent.

`src/main.rs:268-290` -- response is sent as a single message with no length check.

## Fix

Port `split_message()` from the Discord gateway, adapting the character limit to 4096. Split on paragraph boundaries first, then sentence boundaries, then hard-cut at 4096. Send each chunk as a separate sendMessage call.

## Verification

- Send a prompt that produces a response longer than 4096 characters
- Verify the response arrives as multiple messages, each within the limit
- Verify split points occur at natural boundaries (paragraph or sentence) when possible
