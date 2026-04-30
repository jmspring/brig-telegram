---
id: telegram-01
status: open
phase: P2-01
priority: 0
type: bug
tags: [security, ipc, auth]
deps: [telegram-02]
---

# Implement IPC token authentication

BrigHello struct has no `token` field. Gateway uses name `telegram-gateway` which triggers brig's token auth requirement. Connection fails.

## Fix

Add `BRIG_TOKEN` env var, `token` field to BrigHello, include in hello message.

## Cross-reference

- brig-dev: `P2-01-gateway-token-auth.md`
