---
id: telegram-07
status: open
phase: P10-04
priority: 3
type: improvement
tags: [dependencies]
deps: []
---

# Trim ureq features (drop gzip/flate2)

`Cargo.toml`: `ureq = { version = "2", features = ["json"] }` uses default features including gzip/flate2. Main brig crate correctly uses `default-features = false`.

## Fix

```toml
ureq = { version = "2", default-features = false, features = ["tls", "json"] }
```

Saves ~5 transitive deps.

## Cross-reference

- brig-dev: `P10-04-telegram-ureq-features.md`
