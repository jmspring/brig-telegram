---
id: telegram-11
status: done
wave: 12
version: v0.2.0
priority: 3
type: bug
tags: [cli, usability]
deps: []
---
# Write --help to stdout, not stderr

## Problem

`src/main.rs:297-309` -- all help text uses `eprintln!`, writing to stderr. This breaks standard CLI conventions and prevents piping to `less`, `grep`, or other tools.

## Fix

Change `eprintln!` to `println!` for all --help output.

## Verification

- `brig-telegram --help > /tmp/help.txt` captures output (file is non-empty)
- `brig-telegram --help 2>/dev/null` still shows help text
