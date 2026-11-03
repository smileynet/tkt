---
id: "99"
title: "Make update check interval configurable"
status: done
blocked_by: []
validation_criteria:
  - "`update_check.interval` accepted as a user config key"
tags: ["config", "kind:enhancement", "triage:ready-for-agent", "stream:config"]
assignee: "codex"
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

- [x] `update_check.interval` accepted as a user config key
- [x] Supports at minimum: `1h`, `24h`, `7d`, `never`
- [x] Default remains 24h when unconfigured
- [x] `TKT_UPDATE_CHECK=0` still overrides config (env > config > default)
- [x] Existing tests pass unchanged

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `config`.

Deferred enhancement. src/update_check.rs still hardcodes CHECK_INTERVAL_SECS=86400; define duration validation and env precedence without changing default behavior.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.


## Reactivation — 2026-11-03

The owner requested review of all parked work and execution of actionable tickets. This supersedes earlier blanket deferral. See [the parked-work review](../docs/parked-review-2026-11-03.md) for current scope and sequencing. Independent critic review is required before closure.

## Resolution (2026-11-03)

Implemented configurable update cadence with preserved suppression guards; final full gate and independent critic accepted.

### Verification
1. ✓ `update_check.interval` accepted as a user config key — "Public config tests cover defaults, valid/invalid values, dry-run, project rejection, environment precedence and unset. The full gate passed 401 Rust tests and 42 Node fixtures, with one existing Windows skip. See docs/evidence/update-interval-99.md."
