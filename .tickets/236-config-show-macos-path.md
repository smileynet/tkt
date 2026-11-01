---
id: "236"
title: "tkt config shows the wrong user-config path on macOS"
status: open
blocked_by: []
validation_criteria:
  - "tkt config and its help print the path the config loader actually reads on each OS"
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
