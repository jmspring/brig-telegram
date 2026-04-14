---
id: bt-8v0s
status: open
deps: []
links: []
created: 2026-04-14T02:55:55Z
type: feature
priority: 1
assignee: Jim Spring
tags: [multi-bot, config]
---
# Configurable gateway name and scope

## Goal

Make the gateway name and session key prefix configurable via environment variables, so
multiple instances of brig-telegram can run simultaneously with distinct identities.

Memory isolation is per-user, not per-gateway — brig derives scope from the user_id in
the session key (e.g., `tg-{user_id}`). But each gateway instance still needs a unique
name for brig's audit logging and a unique session prefix to avoid session key collisions
when the same user talks to multiple bots.

## Context

Currently in `src/main.rs`, the gateway identity is hardcoded in three places:

1. **Hello handshake name** (line ~126): `name: "telegram-gateway"` — sent to brig in
   the IPC hello message. Used for audit logging and display (not memory scoping —
   memory scope is derived per-user from the session key).

2. **Session key prefix** (line ~312): `format!("tg-{}-{}", chat_id, user_id)` — used
   to track conversation continuity in brig's session database.

3. **Startup log** (line ~270): `eprintln!("brig-telegram starting")` — identifies the
   instance in logs.

To run two Telegram bots (e.g., an ops bot and a support bot), each instance needs a
unique name for audit/logging and a unique session prefix so the same Telegram user
talking to both bots gets separate conversation histories (though their memory facts
are shared — per-user isolation, not per-gateway).

## Required Changes

### 1. Add `BRIG_GATEWAY_NAME` environment variable

Read at startup alongside the existing env vars:

```rust
let gateway_name = env::var("BRIG_GATEWAY_NAME")
    .unwrap_or_else(|_| "telegram-gateway".to_string());
```

Default preserves backward compatibility.

### 2. Use gateway_name in hello handshake

```rust
// Before (line ~124-128):
let hello = BrigHello {
    msg_type: "hello",
    name: "telegram-gateway",
    version: "0.1.0",
};

// After:
let hello = BrigHello {
    msg_type: "hello",
    name: &gateway_name,
    version: "0.1.0",
};
```

Note: `BrigHello` currently uses `&'static str` for `name`. Change to `&str` with a
lifetime, or to `String`. Given that the struct is only used once for serialization,
`String` is simplest.

### 3. Add `BRIG_SESSION_PREFIX` environment variable

```rust
let session_prefix = env::var("BRIG_SESSION_PREFIX")
    .unwrap_or_else(|_| "tg".to_string());
```

Then update session key generation (line ~312):

```rust
// Before:
let session = format!("tg-{}-{}", chat_id, user_id);

// After:
let session = format!("{}-{}-{}", session_prefix, chat_id, user_id);
```

This ensures different bot instances produce different session keys even if the same
Telegram user messages both bots.

### 4. Update startup logging

```rust
eprintln!("{} starting", gateway_name);
eprintln!("  socket: {}", socket_path);
eprintln!("  session prefix: {}", session_prefix);
```

### 5. Update BrigHello struct

The `BrigHello` struct currently uses `&'static str` fields:

```rust
struct BrigHello {
    msg_type: &'static str,
    name: &'static str,
    version: &'static str,
}
```

Change `name` to `String` (or use a lifetime parameter). The simplest approach:

```rust
struct BrigHello<'a> {
    msg_type: &'static str,
    name: &'a str,
    version: &'static str,
}
```

### 6. Update environment variable documentation

Update the table in the README.md and CLAUDE.md to include the new variables:

| Variable              | Required | Default              | Description                           |
|-----------------------|----------|----------------------|---------------------------------------|
| `BRIG_TELEGRAM_TOKEN` | Yes      | --                   | Bot token from @BotFather             |
| `BRIG_SOCKET`         | No       | `/var/brig/sock/...` | Path to Brig's unix socket            |
| `BRIG_GATEWAY_NAME`   | No       | `telegram-gateway`   | Gateway identity for brig (audit/log) |
| `BRIG_SESSION_PREFIX`  | No       | `tg`                 | Session key prefix                    |

## Files to Modify

- `src/main.rs` — env var reading, BrigHello struct, session key generation, logging
- `README.md` — environment variable table
- `CLAUDE.md` — environment variable table (if present)

## Acceptance Criteria

- Default behavior unchanged: `BRIG_GATEWAY_NAME` absent → uses `"telegram-gateway"`
- `BRIG_GATEWAY_NAME=telegram-ops-bot` → hello message sends `name: "telegram-ops-bot"`
- `BRIG_SESSION_PREFIX=tg-ops` → session keys are `tg-ops-{chat}-{user}`
- Two instances with different names and tokens can run simultaneously
- `cargo build` succeeds, `cargo test` passes (if tests exist)
- README updated with new environment variables


## Notes

**2026-04-14T02:58:14Z**

Cross-project dependency: brig tickets bri-6yz7 (scoped facts) and bri-2ks0 (thread user identity) should be completed first for per-user memory isolation to take effect. However, this ticket can be implemented independently — the gateway will send its configured name in the hello message regardless. Memory scope is derived from user_id in the session key, not from the gateway name.
