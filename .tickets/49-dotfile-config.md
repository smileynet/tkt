---
id: "49"
title: "Dotfile config (~/.tkt) for persistent settings"
status: done
blocked_by: []
---

# Dotfile config (~/.tkt) for persistent settings

## What to build

tkt should manage its own dotfile config at `~/.config/tkt/config.toml` (XDG) or `~/Library/Application Support/tkt/config.toml` (macOS) for persistent settings like `debug` and `telemetry`, instead of requiring users to modify their shell profile (`.zshrc`, `.bashrc`).

Commands like `tkt config set debug true` / `tkt config get debug` would read/write this file. On startup, tkt reads the config and applies settings (e.g., enables debug output) without needing env vars.

**Motivation:** Enabling `TKT_DEBUG=1` currently requires editing `~/.zshrc` manually — tkt should own its own configuration.

## Acceptance criteria

- [x] tkt reads a config file from a platform-appropriate location on startup
- [x] `tkt config set <key> <value>` writes to the config file
- [x] `tkt config get <key>` reads from the config file
- [x] `tkt config list` shows all current settings
- [x] `debug = true` in config produces the same behavior as `TKT_DEBUG=1`
- [x] Env vars override config file values (env > config > default)
- [x] Config file is created on first `tkt config set` (not on install)

## Resolution (2026-08-08)

Duplicate of #48 — implemented there. All ACs met.
