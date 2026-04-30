---
id: telegram-06
status: open
phase: P5-06
priority: 2
type: bug
tags: [ipc, timeout]
deps: [telegram-01]
---

# Add socket read/write timeouts

No timeouts on the unix socket. If brig stalls, the gateway blocks forever. Discord has 300s/30s.

## Fix

```rust
stream.set_read_timeout(Some(Duration::from_secs(300)))?;
stream.set_write_timeout(Some(Duration::from_secs(30)))?;
```

## Cross-reference

- brig-dev: `P5-06-gateway-socket-timeouts.md`
