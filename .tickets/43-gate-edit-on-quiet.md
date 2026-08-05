---
id: "43"
title: "Gate edit output on is_quiet() (F5)"
status: open
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

- [ ] `tkt edit <id> --title X -q` produces no stdout
- [ ] `tkt edit <id> --title X` (without -q) still prints the confirmation
- [ ] Decision documented for renumber quiet behavior
