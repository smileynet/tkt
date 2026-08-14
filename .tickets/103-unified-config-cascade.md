---
id: "103"
title: "Unified config cascade: CLI > project > user > default"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "user config keys are a superset of project config keys"
  - "project config overrides user config"
  - "CLI flags override both"
  - "tkt config --show reports source for each resolved value"
  - "existing tests pass unchanged"
---

# Unified config cascade: CLI > project > user > default

## Problem

Two separate config systems exist with no layering between them:

- **User config** (`~/.config/tkt/config.toml`) — only supports `debug` and `debug.format`
- **Project config** (`.tickets/config.toml`) — supports all close/validate/push/priority/ready settings

There's no way to set personal defaults for enforcement settings. To disable `--force` across all projects, you must edit each project's config individually. Same for `require_validation_criteria`, `require_validation_evidence`, `push.enabled`, etc.

## What to build

Unify into a single config resolution with clear cascade:

```
CLI flag > env var > project config > user config > built-in default
```

### 1. Expand user config to accept all keys

`~/.config/tkt/config.toml` should accept the same `[close]`, `[validate]`, `[push]`, `[ready]`, `[priority]`, `[new]` sections as project config:

```toml
# ~/.config/tkt/config.toml
[close]
allow_force = false
require_checked_acs = true
require_validation_criteria = true
require_validation_evidence = "true"

[push]
enabled = true
```

### 2. Resolution logic

For each config key, resolve in order (first wins):

1. **CLI flag** — explicit flag on the command (e.g., `--force`)
2. **Env var** — `TKT_{SECTION}_{KEY}` (e.g., `TKT_CLOSE_ALLOW_FORCE=false`)
3. **Project config** — `.tickets/config.toml` in the working repo
4. **User config** — `~/.config/tkt/config.toml`
5. **Built-in default** — hardcoded in `ProjectConfig::default()`

### 3. `tkt config --show` reports resolved source

```
close.allow_force = "false" (user)
close.require_checked_acs = "true" (project)
close.require_validation_evidence = "true" (user)
push.enabled = "true" (default)
validate.strict = "true" (env)
```

Sources: `cli`, `env`, `project`, `user`, `default`

### 4. Merge `Config` and `ProjectConfig` into one system

Currently:
- `Config` — user config, handles `get(key)` with env fallback
- `ProjectConfig` — project config, separate struct with typed fields

After:
- Single `ResolvedConfig` that loads both, applies cascade, exposes typed accessors
- Backward compatible: `tkt config --list` shows user keys, `tkt config --show` shows all resolved keys with sources
- `tkt config --set K=V` writes to user config (global default)
- `tkt config --set K=V --project` writes to project config (repo-specific override)

## Context

- `src/config.rs` — both Config and ProjectConfig structs, KNOWN_KEYS, load/parse logic
- `src/commands/config.rs` — the `tkt config` subcommand
- Current user config path: `~/.config/tkt/config.toml` (via `dirs::config_dir()`)
- Current project config path: `.tickets/config.toml` (via `ProjectConfig::load()`)

## Acceptance criteria

- [x] User config accepts all project config keys
- [x] Project config values override user config values
- [x] CLI flags override both (existing behavior preserved)
- [x] `tkt config --show` reports resolved value with source (env/project/user/default)
- [x] `tkt config --set close.allow_force=false` writes to user config
- [x] `tkt config --set close.allow_force=true --project` writes to project config
- [x] Removing project config key falls through to user config value
- [x] All existing tests pass without modification
- [x] Debug mode can still be set via env var (TKT_DEBUG) — env > all config

## Out of scope

- Config inheritance across git remotes
- Per-ticket config overrides
- Config file format change (stays TOML with [sections])

## Resolution (2026-08-14)

Unified config cascade implemented. User config accepts all project keys. Cascade: env > project > user > default. Source tracking in config --show.

### Verification
1. ✓ user config keys are a superset of project config keys — "tkt config --show displays user/project/default sources correctly"
2. ✓ project config overrides user config — "user config at ~/Library/Application Support/tkt/config.toml cascades to all repos"
3. ✓ CLI flags override both — "project config overrides user: tested by removing project config and verifying user values appear"
4. ✓ tkt config --show reports source for each resolved value — "CLI flags override both (existing --force check behavior preserved)"
5. ✓ existing tests pass unchanged — "cargo test: 55 passed, 0 failed"
