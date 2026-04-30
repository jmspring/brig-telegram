---
id: telegram-15
status: done
wave: 12
version: v0.2.0
priority: 4
type: bug
tags: [cli, security, correctness]
deps: [telegram-14]
---
# BRIG_TOKEN marked "required" in help but actually optional

## Problem

`src/main.rs:303` -- the `--help` text says BRIG_TOKEN is "required", but the code only emits a warning when it is missing and continues to connect without authentication. The help text and runtime behavior contradict each other.

## Fix

Either enforce the requirement (exit with an error if BRIG_TOKEN is unset) or update the help text to say it is optional with a warning about unauthenticated connections.

## Verification

- If enforced: `unset BRIG_TOKEN && brig-telegram` exits with a clear error
- If optional: help text no longer says "required" and explains the security implication
- Help text and runtime behavior are consistent
