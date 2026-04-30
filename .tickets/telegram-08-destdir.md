---
id: telegram-08
status: open
phase: P8-06
priority: 3
type: bug
tags: [freebsd, makefile, packaging]
deps: []
---

# Add DESTDIR support to Makefile

`Makefile:23` uses `${PREFIX}` directly. Need `${DESTDIR}${PREFIX}` for staged installs.

## Cross-reference

- brig-dev: `P8-06-gateway-makefiles.md`
