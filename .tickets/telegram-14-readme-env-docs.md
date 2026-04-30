---
id: telegram-14
status: done
wave: 12
version: v0.2.0
priority: 3
type: docs
tags: [documentation, security]
deps: []
---
# Document BRIG_TOKEN and BRIG_TELEGRAM_ALLOWED_USERS in README

## Problem

`BRIG_TOKEN` is not listed in the README environment variable table. `BRIG_TELEGRAM_ALLOWED_USERS` only appears in `--help` output. Both are security-critical for a single-operator tool and should be discoverable in the primary documentation.

## Fix

Add both variables to the README.md environment variable table with descriptions, default behavior, and security implications.

## Verification

- README.md contains entries for both BRIG_TOKEN and BRIG_TELEGRAM_ALLOWED_USERS
- Each entry describes the variable's purpose, format, and what happens when omitted
