---
id: telegram-02
status: open
phase: P1-04
priority: 0
type: security
tags: [freebsd, rc.d, secrets]
deps: []
---

# Fix bot token exposure in rc.d script

`scripts/rc.d/brig_telegram:38`: Passes Telegram bot token via `env` in `command_args`, visible in `ps -auxww`.

## Fix

Write token to mode-0600 env file in `start_precmd`, use `daemon -E` to source it.

## Cross-reference

- brig-dev: `P1-04-rcd-token-exposure.md`
