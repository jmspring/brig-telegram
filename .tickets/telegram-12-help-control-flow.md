---
id: telegram-12
status: done
wave: 12
version: v0.2.0
priority: 4
type: quality
tags: [cli, control-flow]
deps: [telegram-11]
---
# Fix --help control flow (process::exit inside run())

## Problem

`src/main.rs:296-314` -- the `--help` and `--version` flags are handled inside `run()`, which returns `Result`. Instead of returning, they call `process::exit(0)`, giving the function two exit mechanisms (return and exit). This makes control flow deceptive and prevents callers from doing cleanup.

## Fix

Either return early from `run()` with an Ok variant, or move help/version handling to `main()` before calling `run()`.

## Verification

- `brig-telegram --help` still prints help and exits 0
- `brig-telegram --version` still prints version and exits 0
- No `process::exit` calls remain inside `run()` for help/version paths
