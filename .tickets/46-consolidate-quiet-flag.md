---
id: "46"
title: "Consolidate quiet flag mechanisms (F8)"
status: done
blocked_by: []
---

# Consolidate quiet flag mechanisms (F8)

## Origin

Review ticket #38, finding F8.

## Problem

Two parallel mechanisms exist for the quiet flag:
1. `static QUIET: AtomicBool` + `is_quiet()` function (used by most commands)
2. `cmd_ready(json, quiet)` takes quiet as a parameter

Both are set from `cli.quiet`. A future command could read one and miss the other.

## What to build

Keep one mechanism. Since the arg is `global = true`, the `AtomicBool` + `is_quiet()` pattern is the right one. Remove the `quiet` parameter from `cmd_ready` and have it call `is_quiet()` like everything else.

## Acceptance criteria

- [x] `cmd_ready` uses `is_quiet()` instead of a parameter
- [x] No function takes `quiet: bool` as a parameter
- [x] `tkt ready -q` still works correctly
- [x] All tests pass

## Resolution (2026-08-05)

Removed `quiet` parameter from `cmd_ready`, inlined `cli.quiet` store into the global AtomicBool, changed the one remaining bare `quiet` reference to `is_quiet()`. All 69 tests pass.
