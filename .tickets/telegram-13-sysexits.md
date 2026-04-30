---
id: telegram-13
status: done
wave: 12
version: v0.2.0
priority: 3
type: feature
tags: [cli, bsd-conventions]
deps: []
---
# Use sysexits.h exit codes

## Problem

`src/main.rs:440` -- the binary exits with code 1 for every error class, whether it is a usage error, a missing config, or an unavailable service. This prevents scripts and rc.d from distinguishing failure modes.

## Fix

Map error classes to sysexits codes: EX_USAGE (64) for bad arguments, EX_DATAERR (65) for config parse errors, EX_UNAVAILABLE (69) for socket/network failures, EX_CONFIG (78) for missing config. Keep EX_SOFTWARE (70) as the catch-all.

## Verification

- `brig-telegram --bad-flag; echo $?` prints 64
- `BRIG_SOCKET=/nonexistent brig-telegram; echo $?` prints 69 (or appropriate code)
- Valid invocation with working config exits 0 on clean shutdown
