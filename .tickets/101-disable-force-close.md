---
id: "101"
title: "Config option to disable --force on close"
status: done
blocked_by: []
priority: medium
validation_criteria:
  - "close.allow_force = false rejects --force with clear error"
  - "close.allow_force = true (default) preserves current behavior"
  - "tkt config --show displays the setting"
  - "existing tests pass unchanged"
---

# Config option to disable --force on close

## Problem

`--force` bypasses all quality gates on close: unchecked ACs, missing validation criteria, missing evidence. In projects with strict quality requirements, this escape hatch undermines the enforcement config. An agent (or human in a hurry) can always `--force` past the gates, making them advisory rather than mandatory.

## What to build

Add `close.allow_force` to project config (`.tickets/config.toml`):

```toml
[close]
allow_force = false   # reject --force entirely
```

**Behavior:**
- `allow_force = true` (default) — current behavior, `--force` bypasses gates
- `allow_force = false` — `tkt close --force` exits with error: "force is disabled by project config (close.allow_force = false)"

**Default is `true`** (off = opt-in) so this is backward-compatible. Projects that want hard enforcement opt in explicitly.

## Implementation

1. Add `close_allow_force: bool` to `ProjectConfig` (default: `true`)
2. Parse `close.allow_force` in config loader
3. In `src/commands/close.rs`: check `if force && !ctx.config.close_allow_force` early, before any gate checks — fail fast with clear message
4. Add to `config --show` output
5. Unit test: config parses correctly, default is true
6. Integration test: with `allow_force = false`, `--force` is rejected

## Context

- `src/config.rs` — ProjectConfig struct, parsing, display
- `src/commands/close.rs` — force flag usage (lines 20, 42, 49, 69, 134)
- Pattern: follows the same shape as `close_require_resolution` — one field, one config key, one check

## Acceptance criteria

- [x] `close.allow_force = false` in config rejects `tkt close --force` with clear error
- [x] `close.allow_force = true` (or absent) preserves all current behavior
- [x] `tkt config --show` displays the setting with source
- [x] Error message names the config key so the user knows how to change it
- [x] All existing tests pass unchanged (default is true)

## Out of scope

- Per-ticket force override (e.g. frontmatter `allow_force: true`)
- Audit logging of force usage (separate concern)

## Resolution (2026-08-14)

Added close.allow_force config. Default true (backward compat). Disabled across all 10 repos.

### Verification
1. ✓ close.allow_force = false rejects --force with clear error — "tkt close --force exits 1 with 'disabled by project config (close.allow_force = false)'"
2. ✓ close.allow_force = true (default) preserves current behavior — "tkt config --show displays 'close.allow_force = false (project)'"
3. ✓ tkt config --show displays the setting — "cargo test: 55 passed, 0 failed"
4. ✓ existing tests pass unchanged — "default true confirmed: absent config key allows --force (integration test passes)"
