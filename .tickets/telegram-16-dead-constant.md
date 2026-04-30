---
id: telegram-16
status: done
wave: 14
version: v0.3.0
priority: 4
type: quality
tags: [code-quality, dead-code]
deps: []
---
# Remove dead DEFAULT_SOCKET constant

## Problem

`src/main.rs:14` -- `const DEFAULT_SOCKET` is defined but the same value is inlined in the socket resolution logic at line 335. Other gateways inline the path without a constant. The constant creates a false sense of single-source-of-truth when the value is actually duplicated.

## Fix

Remove the constant and inline the fallback value at the usage site, or use the constant consistently by replacing the inlined duplicate. Either way, eliminate the duplication.

## Verification

- `cargo build` succeeds with no warnings
- `grep -n DEFAULT_SOCKET src/main.rs` shows either zero hits (removed) or exactly one definition and one usage (consolidated)
