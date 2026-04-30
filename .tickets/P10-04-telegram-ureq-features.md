# P10-04: Trim brig-telegram ureq features

**Phase:** 10 — Dependency Optimization
**Severity:** MEDIUM
**Effort:** S (<5min)
**Component:** brig-telegram
**Personas:** 1/7 (rust-minimal-deps)
**Depends on:** none
**Blocks:** none

## Problem

`brig-telegram/Cargo.toml`: `ureq = { version = "2", features = ["json"] }` uses default features, which includes `gzip`/`flate2`. The main brig crate correctly uses `default-features = false, features = ["tls", "json"]`. Dropping defaults removes ~5 transitive deps.

## Files to change

- `brig-telegram/Cargo.toml`

## Fix

```toml
# Before:
ureq = { version = "2", features = ["json"] }
# After:
ureq = { version = "2", default-features = false, features = ["tls", "json"] }
```

## Verification

- `cargo build` succeeds
- Telegram API calls still work (TLS required for api.telegram.org)
- ~5 fewer transitive deps
