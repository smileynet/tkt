---
id: "147"
title: "Investigate telemetry consent resetting to disabled between sessions"
status: backlog
blocked_by: []
priority: medium
validation_criteria:
  - "root cause identified and fixed or documented"
tags: ["config", "kind:bug", "triage:needs-info", "stream:config", "gate:evidence"]
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

## Planning gate

Capture platform, configuration paths and commands that demonstrate consent reverting.

## Cross-project evidence review — 2026-11-03

A read-only review of 78 other project boards and attributed operational reports found no enable/restart/config-path sequence proving a consent reset. Retained telemetry contains only 27 events and no consent-transition proof; it cannot establish historical absence. Keep backlog pending the exact reproduction, without changing consent. See [the evidence review](../docs/evidence/cross-project-review-2026-11-03.md).

## Note — 2026-11-04

By: anonymous

<!-- tkt-note:v1 101860-1793793399448958400 -->
2026-11-04 reproduction attempt: exact current consent reader in private fresh processes preserves enabled bytes across five restarts. DNT=1, tool opt-out, present CI (even empty), or an alternate config root explain effective disabled without a persistent reset. Injected private reset control was caught; actual private writer/preview tests pass2. This is not whole-CLI or original historical reproduction. Keep evidence gate/backlog; no real user consent changed. Probe/results and required failing-sequence evidence: docs/evidence/remaining-six-review-2026-11-04.md.
