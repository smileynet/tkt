---
id: "99"
title: "Make update check interval configurable"
status: backlog
blocked_by: []
validation_criteria:
  - "`update_check.interval` accepted as a user config key"
tags: ["config", "kind:enhancement", "triage:ready-for-agent", "stream:config"]
---

# Make update check interval configurable

## Context

The update check (ticket 45) is implemented and works: checks crates.io once per 24 hours, 3s timeout, prints to stderr. Can be disabled entirely via `TKT_UPDATE_CHECK=0` or in CI.

What's missing: the 24h interval is hardcoded. Users should be able to tune it or disable it through `tkt config` rather than env vars.

## What to build

Add a user config key `update_check.interval` (or `update_check = "24h" | "never" | "7d"`):

- `tkt config --set update_check.interval=24h` — check once per day (default, current behavior)
- `tkt config --set update_check.interval=7d` — check once per week
- `tkt config --set update_check.interval=never` — disable (equivalent to TKT_UPDATE_CHECK=0)

Parse duration from the config, replace the hardcoded `CHECK_INTERVAL_SECS` constant with the configured value.

## Acceptance criteria

- [ ] `update_check.interval` accepted as a user config key
- [ ] Supports at minimum: `1h`, `24h`, `7d`, `never`
- [ ] Default remains 24h when unconfigured
- [ ] `TKT_UPDATE_CHECK=0` still overrides config (env > config > default)
- [ ] Existing tests pass unchanged

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `config`.

Deferred enhancement. src/update_check.rs still hardcodes CHECK_INTERVAL_SECS=86400; define duration validation and env precedence without changing default behavior.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.
