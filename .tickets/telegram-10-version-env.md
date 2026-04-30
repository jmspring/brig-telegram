---
id: telegram-10
status: done
wave: 12
version: v0.2.0
priority: 3
type: bug
tags: [correctness, build]
deps: []
---
# Replace hardcoded version "0.1.0" with env!()

## Problem

`src/main.rs:163` -- the `BrigHello` version field hardcodes `"0.1.0"` while `Cargo.toml` is at version 0.1.3. The daemon receives stale version info on every connection.

## Fix

Replace the hardcoded string with `env!("CARGO_PKG_VERSION")` so the version stays in sync with Cargo.toml automatically.

## Verification

- `cargo build` succeeds
- Grep the binary for "0.1.3" (or current Cargo.toml version) to confirm it is embedded
- Confirm no remaining hardcoded "0.1.0" in src/main.rs outside of comments
