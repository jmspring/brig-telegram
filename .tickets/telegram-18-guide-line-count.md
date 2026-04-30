---
id: telegram-18
status: done
wave: 15
version: v0.4.0
priority: 4
type: docs
tags: [documentation, accuracy]
deps: []
---
# GUIDE line count claim wrong

## Problem

The GUIDE claims the gateway is "~350 lines" but the actual file is 443 lines. The claim is misleading for anyone evaluating the codebase scope.

## Fix

Update the line count to match reality, or remove the specific claim entirely.

## Verification

- `wc -l src/main.rs` output is consistent with whatever the GUIDE states
- Or the GUIDE no longer makes a line count claim
