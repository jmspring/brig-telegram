---
id: telegram-04
status: open
phase: P5-03
priority: 1
type: security
tags: [security, auth, telegram]
deps: [telegram-01]
---

# Add user allowlist

Any Telegram user who discovers the bot can submit tasks. Add `BRIG_TELEGRAM_ALLOWED_USERS` env var (comma-separated user IDs).

## Fix

Parse env var at startup. Filter messages in the processing loop — silently drop messages from users not on the list. Without env var set, all users accepted (backwards compatible).

## Cross-reference

- brig-dev: `P5-03-telegram-user-allowlist.md`
