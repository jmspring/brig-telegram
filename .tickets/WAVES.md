# brig-telegram Ticket Waves

## Pre-wave (Phase-based, April 18)

Original tickets from cross-project phase review. Phase IDs retained for traceability.

| Ticket | Phase | Title | Priority | Status |
|--------|-------|-------|----------|--------|
| telegram-01 | P2-01 | Token auth for IPC socket | 0 (critical) | open |
| telegram-02 | P1-04 | rc.d token exposure | 0 (critical) | open |
| telegram-03 | P2-03 | --help/--version flags | 3 | open |
| telegram-04 | P5-03 | User allowlist | 2 | open |
| telegram-05 | P5-05 | Bounded reads | 2 | open |
| telegram-06 | P5-06 | Socket timeouts | 2 | open |
| telegram-07 | P10-04 | ureq feature trimming | 3 | open |
| telegram-08 | P8-06 | DESTDIR support | 3 | open |

### Consolidated phase tickets

| Ticket | Title | Priority | Status |
|--------|-------|----------|--------|
| P5-03 | User allowlist (cross-ref telegram-04) | HIGH | open |
| P10-04 | ureq features (cross-ref telegram-07) | MEDIUM | open |

### Standalone tickets

| Ticket | Title | Priority | Status |
|--------|-------|----------|--------|
| bt-8v0s | Configurable gateway name | 1 | open |
| bt-hxj5 | Multi-instance manifest | 2 | open |
| bt-p0rt | FreeBSD port | 2 | done |

---

## Wave 12 (v0.2.0) -- Reliability & Features

| Ticket | Title | Priority | Type | Effort |
|--------|-------|----------|------|--------|
| telegram-09 | Message splitting for long responses | 2 (high) | bug | S |
| telegram-10 | Replace hardcoded version with env!() | 3 (medium) | bug | S |
| telegram-11 | Write --help to stdout | 3 (medium) | bug | S |
| telegram-12 | Fix --help control flow | 4 (low) | quality | S |
| telegram-13 | Use sysexits.h exit codes | 3 (medium) | feature | S |
| telegram-14 | Document BRIG_TOKEN and BRIG_TELEGRAM_ALLOWED_USERS | 3 (medium) | docs | S |
| telegram-15 | BRIG_TOKEN required/optional mismatch | 4 (low) | bug | S |

**Dependencies within wave:**
- telegram-12 depends on telegram-11 (stdout fix first, then control flow)
- telegram-15 depends on telegram-14 (document behavior first, then fix inconsistency)

---

## Wave 14 (v0.3.0) -- Code Quality

| Ticket | Title | Priority | Type | Effort |
|--------|-------|----------|------|--------|
| telegram-16 | Remove dead DEFAULT_SOCKET constant | 4 (low) | quality | S |

---

## Wave 15 (v0.4.0) -- Documentation

| Ticket | Title | Priority | Type | Effort |
|--------|-------|----------|------|--------|
| telegram-17 | Fix `brig skill add` syntax in GUIDE | 3 (medium) | docs | S |
| telegram-18 | GUIDE line count claim wrong | 4 (low) | docs | S |
