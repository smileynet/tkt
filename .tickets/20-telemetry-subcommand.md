---
id: "20"
title: "tkt telemetry subcommand (enable/disable/status/show)"
status: done
blocked_by: ["18"]
---

# tkt telemetry subcommand

## What to build

Add a `tkt telemetry` subcommand following the Turbo/Next.js pattern for managing consent and inspecting collected data.

### Commands

```
tkt telemetry --enable      # persist opt-in to consent.toml
tkt telemetry --disable     # persist opt-out
tkt telemetry --status      # show: enabled/disabled, reason (env var / config / default), storage used
tkt telemetry --show        # dump local JSONL events (human-readable summary)
tkt telemetry --clear       # delete all local telemetry data
```

### Status output

```
telemetry: disabled (default — never opted in)
storage: 0 bytes (0 events across 0 projects)
consent file: ~/.config/tkt/consent.toml (not found)
env overrides: DO_NOT_TRACK=unset, TKT_TELEMETRY=unset, CI=unset
```

Or when enabled:
```
telemetry: enabled (consent.toml)
storage: 142 KB (847 events across 3 projects)
  tkt: 312 events (68 KB)
  game-research: 401 events (52 KB)
  shadowrun-sega: 134 events (22 KB)
consent file: ~/.config/tkt/consent.toml
env overrides: DO_NOT_TRACK=unset, TKT_TELEMETRY=unset, CI=unset
```

### Consent file format

```toml
# ~/.config/tkt/consent.toml
[telemetry]
enabled = true
consented_at = "2026-07-30"
version = 1
```

### Deletion test

Without this subcommand, users have no way to opt in, inspect what's collected, or manage storage.

## Acceptance criteria

- [x] `tkt telemetry --enable` writes consent.toml and confirms
- [x] `tkt telemetry --disable` writes consent.toml and confirms
- [x] `tkt telemetry --status` shows current state with reason and storage summary
- [x] `tkt telemetry --show` prints a human-readable summary of recent events
- [x] `tkt telemetry --clear` deletes all telemetry files and confirms
- [x] Exit code 0 for all telemetry subcommands
- [x] Integration test covering enable → status → disable → status cycle
