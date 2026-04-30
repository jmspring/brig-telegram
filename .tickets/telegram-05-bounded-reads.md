---
id: telegram-05
status: open
phase: P5-05
priority: 2
type: bug
tags: [security, ipc]
deps: [telegram-01]
---

# Add bounded reads on brig socket

`main.rs:156`: Unbounded `read_line` — OOM vector. Implement `read_line_bounded()` with 1MB max.

## Cross-reference

- brig-dev: `P5-05-gateway-bounded-reads.md`
