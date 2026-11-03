---
id: "147"
title: "Investigate telemetry consent resetting to disabled between sessions"
status: backlog
blocked_by: []
priority: medium
validation_criteria:
  - "root cause identified and fixed or documented"
tags: ["config", "kind:bug", "triage:needs-info"]
---

# Investigate telemetry consent resetting to disabled between sessions

## What to build

Re-triage 2026-10-29: no reproduction was ever recorded. Before any fix, capture the
platform, config file paths, the consent command used and the sequence that shows
consent reverting.

## Acceptance criteria

- [ ] Reproduction steps are recorded, then the root cause is fixed or documented

## Triage — 2026-11-02

Disposition: `kind:bug`, `triage:needs-info`; stream `config`.

No platform/config/consent reproduction is recorded. Deferred investigation; do not change consent precedence speculatively or assert ambient consent state.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.
