---
id: bt-hxj5
status: open
deps: [bt-8v0s]
links: []
created: 2026-04-14T02:56:39Z
type: feature
priority: 2
assignee: Jim Spring
tags: [multi-bot, deployment, manifest]
---
# Multi-instance manifest and service support

## Goal

Document and support the pattern for running multiple brig-telegram instances as
separate brig persistent skills, each with its own manifest, rc.d service, jail,
secret, and gateway identity.

## Context

After the configurable gateway name ticket (bt-8v0s), brig-telegram supports
`BRIG_GATEWAY_NAME` and `BRIG_SESSION_PREFIX` env vars. But to actually deploy two
Telegram bots, the operator needs two separate persistent skill registrations in brig,
each with its own manifest, secret, and service name.

The current `manifest.toml` is a single file with hardcoded values. This ticket creates
a pattern for multi-instance deployment.

**Depends on:** "Configurable gateway name and scope" (bt-8v0s).

## Required Changes

### 1. Create `contrib/` directory with example manifests

Create example manifests for multi-instance deployment:

`contrib/manifest-ops.toml`:
```toml
[skill]
name = "telegram-ops-bot"
description = "Telegram gateway for ops team"
kind = "persistent"

[requires]
network = ["api.telegram.org"]
max_runtime = "forever"

[persistent]
rc_name = "brig_tg_ops"
entrypoint = "/usr/local/bin/brig-telegram"
restart_on_failure = true
depends_on = ["brig"]
env = { BRIG_GATEWAY_NAME = "telegram-ops-bot", BRIG_SESSION_PREFIX = "tg-ops" }

[persistent.socket]
capabilities = ["submit_task", "read_status"]

[secrets]
telegram_token = { env = "BRIG_TELEGRAM_TOKEN" }
```

`contrib/manifest-support.toml`:
```toml
[skill]
name = "telegram-support-bot"
description = "Telegram gateway for customer support"
kind = "persistent"

[requires]
network = ["api.telegram.org"]
max_runtime = "forever"

[persistent]
rc_name = "brig_tg_support"
entrypoint = "/usr/local/bin/brig-telegram"
restart_on_failure = true
depends_on = ["brig"]
env = { BRIG_GATEWAY_NAME = "telegram-support-bot", BRIG_SESSION_PREFIX = "tg-support" }

[persistent.socket]
capabilities = ["submit_task", "read_status"]

[secrets]
telegram_token = { env = "BRIG_TELEGRAM_TOKEN" }
```

### 2. Verify brig's persistent skill `env` support

The manifests above use `[persistent] env = { ... }` to pass environment variables to
the entrypoint. Verify that brig's skill enable/rc.d generation actually reads and
injects these env vars. If not, this is a dependency on a brig-side change (document
it but don't block on it — operators can set env vars in the rc.d script manually).

### 3. Document multi-instance deployment in docs/GUIDE.md

Add a "Running Multiple Bots" section to the guide covering:

1. Register each bot with @BotFather (different bot, different token)
2. Create a manifest per bot (copy and customize from contrib/)
3. Install the single binary once (`/usr/local/bin/brig-telegram`)
4. Register each manifest as a separate skill:
   ```sh
   brig skill add --manifest contrib/manifest-ops.toml
   brig skill add --manifest contrib/manifest-support.toml
   ```
5. Set secrets for each:
   ```sh
   brig secret set telegram-ops-bot.telegram_token
   brig secret set telegram-support-bot.telegram_token
   ```
6. Enable and start each:
   ```sh
   sudo brig skill enable telegram-ops-bot
   sudo brig skill enable telegram-support-bot
   sudo service brig_tg_ops start
   sudo service brig_tg_support start
   ```
7. Explain that each bot has isolated memory in brig (facts scoped to its gateway name)
8. Explain that session keys are distinct (different prefix)

### 4. Update README.md

Add a brief note pointing to the multi-instance section in docs/GUIDE.md.

## Files to Create

- `contrib/manifest-ops.toml` — example ops bot manifest
- `contrib/manifest-support.toml` — example support bot manifest

## Files to Modify

- `docs/GUIDE.md` — add "Running Multiple Bots" section
- `README.md` — add cross-reference to multi-instance docs

## Acceptance Criteria

- Example manifests exist in `contrib/` with distinct skill names, rc_names, and env vars
- docs/GUIDE.md has a complete multi-instance deployment walkthrough
- The two example manifests can both be registered with `brig skill add` without conflict
- Each instance uses a different gateway name and session prefix
- Documentation explains memory isolation between bots

