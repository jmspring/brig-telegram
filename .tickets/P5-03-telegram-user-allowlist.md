# P5-03: Add user allowlist to Telegram gateway

**Phase:** 5 — Gateway Hardening
**Severity:** HIGH
**Effort:** S (~30min)
**Component:** brig-telegram
**Personas:** 2/7 (security, adversarial-llm)
**Depends on:** P2-01 (gateway token auth)
**Blocks:** none

## Problem

`brig-telegram/src/main.rs`: Any Telegram user who discovers the bot can submit tasks. There is no user ID or chat ID filtering. With capability enforcement now working, attackers can only trigger skill execution (not admin commands), but this still enables unauthorized resource consumption and indirect prompt injection.

## Files to change

- `brig-telegram/src/main.rs` — add `BRIG_TELEGRAM_ALLOWED_USERS` env var parsing and message filtering
- `brig-telegram/README.md` — document the env var

## Fix

```rust
let allowed_users: Option<Vec<i64>> = std::env::var("BRIG_TELEGRAM_ALLOWED_USERS")
    .ok()
    .map(|s| s.split(',').filter_map(|id| id.trim().parse().ok()).collect());

// In the message processing loop:
if let Some(ref allowed) = allowed_users {
    if !allowed.contains(&message.from.id) {
        continue; // silently ignore unauthorized users
    }
}
```

## Verification

- With `BRIG_TELEGRAM_ALLOWED_USERS=12345,67890`, only those user IDs can submit tasks
- Without the env var set, all users can submit (backwards compatible)
- Unauthorized messages are silently dropped (no error to the user)
