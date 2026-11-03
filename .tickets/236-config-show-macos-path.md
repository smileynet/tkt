---
id: "236"
title: "tkt config shows the wrong user-config path on macOS"
status: open
blocked_by: []
validation_criteria:
  - "tkt config and its help print the path the config loader actually reads on each OS"
tags: ["config", "kind:bug", "triage:ready-for-agent"]
---

# tkt config shows the wrong user-config path on macOS

## Problem

`tkt config` and `tkt config --help` label the user file `~/.config/tkt/config.toml`. On macOS
`src/config.rs` reads `~/Library/Application Support/tkt/config.toml` (via `dirs::config_dir()`,
unless `XDG_CONFIG_HOME` is set).

## Context

Found 2026-11-01 in a philippines-trip retro: `~/.config/tkt/config.toml` did not exist, and the
real file under `~/Library/Application Support/tkt/` held `debug = "true"`. Related done work:
#49 (dotfile config), #103 (unified config cascade).

## Acceptance criteria

- [ ] `tkt config` and its help print the path the config loader actually reads on each OS.

## Triage — 2026-11-02

Disposition: `kind:bug`, `triage:ready-for-agent`; stream `config`.

Confirmed with isolated XDG_CONFIG_HOME: config still labels ~/.config/tkt/config.toml instead of the file actually loaded. Cross-platform loader uses dirs::config_dir; macOS runtime remains untested here.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.
