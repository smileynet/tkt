---
id: "43"
title: "Gate edit output on is_quiet() (F5)"
status: done
blocked_by: []
---

# Gate edit output on is_quiet() (F5)

## Origin

Review ticket #38, finding F5.

## Problem

`tkt edit <id> --title X -q` prints `✓ edited 02 beta (title)` — the `cmd_edit` function's final `println!` is not wrapped in `if !is_quiet()`. Every other mutation command respects the quiet flag.

## What to build

1. Wrap `cmd_edit`'s success println in `if !is_quiet()`
2. Decide whether `cmd_renumber` should also be silent in quiet mode (it's not in #33's table but the principle is the same)

## Acceptance criteria

- [x] `tkt edit <id> --title X -q` produces no stdout
- [x] `tkt edit <id> --title X` (without -q) still prints the confirmation
- [x] Decision documented for renumber quiet behavior

## Resolution (2026-08-05)

Wrapped `cmd_edit` and `cmd_renumber` success output in `if !is_quiet()`. Decision: renumber follows the same principle — all mutation commands are silent in quiet mode. Verified in scratch repo.
