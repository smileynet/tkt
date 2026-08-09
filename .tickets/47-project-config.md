---
id: "47"
title: "per-project config: .tickets/config.toml for tunable behaviors"
status: done
blocked_by: []
---

# Per-project config: .tickets/config.toml for tunable behaviors

## Problem

tkt has behaviors that different projects want to tune:
- Should `close` require resolution text? (crew-research: yes, scratch projects: no)
- Should `validate` default to strict? (CI: yes, local dev: no)
- What's the default env filter? (some repos are always corp, others always personal)
- Should unknown priority values warn or silently pass?

Currently all behavior is hardcoded or controlled by env vars/flags. There's no way to set project-level defaults that travel with the repo.

## Proposed: `.tickets/config.toml`

```toml
# .tickets/config.toml — project-level tkt configuration
# Committed to the repo, shared by all contributors/agents

[close]
require_resolution = false   # error if --resolution not provided (default: false)
require_checked_acs = true   # error if all ACs unchecked without --force (default: true)

[validate]
strict = false               # default strictness (default: false)

[ready]
default_env = ""             # pre-filter by env without CREW_ENV (default: "" = all)

[priority]
warn_unknown = true          # warn on unrecognized priority values in validate (default: true)

[new]
default_priority = ""        # auto-assign priority to new tickets (default: "" = none)

[push]
enabled = true               # set false for local-only repos (skip all push attempts)
```

## Design principles

- **Convention over configuration** — every field has a sensible default. Config file is optional.
- **Committed to repo** — `.tickets/config.toml` is version-controlled, so all contributors/agents share the same settings
- **No user-level config for behavior** — user config (`~/.config/tkt/`) is only for consent and update-check state, not for tool behavior (avoids "works on my machine" divergence)
- **Flags override config** — `tkt validate --strict` overrides `strict = false` in config. Config is the default, not the ceiling.
- **Discoverable** — `tkt config` (or `tkt config --show`) dumps effective configuration with sources

## Discovery behavior

When tkt starts, it looks for `.tickets/config.toml` relative to the repo root. If not found, all defaults apply. No cascading (no user-level behavior overrides — that's what env vars are for).

## Migration

Existing projects without config.toml continue working identically (all defaults match current behavior). Adding a config.toml is purely opt-in enhancement.

## Research questions

- Should there be a `tkt init-config` that generates a commented config.toml?
- Should invalid config keys produce a warning or error? (Recommendation: warning — forward-compatible with newer tkt versions)
- Should config support `[hooks]` for pre-close/pre-push validation scripts?

## Acceptance criteria

- [x] `.tickets/config.toml` parsed at startup when present
- [x] All documented fields supported with correct defaults
- [x] Missing config file = all defaults (no error)
- [x] Unknown keys produce a warning (forward-compatible)
- [x] Flags override config values
- [x] `tkt config --show` dumps effective configuration with source annotations
- [x] Integration test: config changes behavior (e.g., require_resolution = true blocks bare close)
- [x] Document config format in README

## Resolution (2026-08-09)

Implemented: ProjectConfig struct with 7 settings across 5 sections. Parsed from .tickets/config.toml with section headers. All fields have sensible defaults. Unknown keys warn. Flags override config. --show dumps effective settings. 6 tests (2 unit + 4 integration).
