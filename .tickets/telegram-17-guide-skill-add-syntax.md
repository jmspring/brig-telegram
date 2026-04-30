---
id: telegram-17
status: done
wave: 15
version: v0.4.0
priority: 3
type: docs
tags: [documentation, accuracy]
deps: []
---
# Fix `brig skill add` syntax in GUIDE

## Problem

`docs/GUIDE.md:456-457` -- the guide shows `brig skill add --manifest <file>` but the actual CLI takes a directory path, not a `--manifest` flag with a file argument.

## Fix

Update the example to use the correct directory path syntax that matches the actual CLI.

## Verification

- `grep -n 'skill add' docs/GUIDE.md` shows correct syntax
- The documented syntax matches `brig skill add --help` output
