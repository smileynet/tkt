---
id: "145"
title: "Fix update_check ignores DO_NOT_TRACK"
status: in_progress
blocked_by: []
priority: low
validation_criteria:
  - "update check respects DO_NOT_TRACK=1"
tags: ["cli", "kind:bug", "triage:needs-info", "stream:quality", "gate:evidence"]
assignee: "codex"
---

# Fix update_check ignores DO_NOT_TRACK

## What to build

`src/update_check.rs` (main `980b043`) returns early for quiet mode, `TKT_UPDATE_CHECK=0`
and `CI`, but not for `DO_NOT_TRACK=1`, while `telemetry.rs` treats `DO_NOT_TRACK=1` as
disabled. The update check contacts crates.io, so someone who opted out of tracking still
makes the request. Skip the check when `DO_NOT_TRACK=1`, using the same parsing as
telemetry, and document it.

## Acceptance criteria

- [ ] With `DO_NOT_TRACK=1`, no update-check request is made and the cache is not refreshed
- [ ] A test covers `DO_NOT_TRACK=1` alongside `TKT_UPDATE_CHECK=0` and `CI`
- [ ] README and telemetry docs list `DO_NOT_TRACK` as disabling the update check

## Triage — 2026-11-02

Disposition: `kind:bug`, `triage:needs-info`; stream `cli`.

Source inspection confirms update_check.rs has no DO_NOT_TRACK guard. Network/cache behavior was not dynamically reproduced; capture an isolated curl/cache fixture before implementing. Existing privacy request remains actionable investigation.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Reproduce request/cache behavior in an isolated fixture before changing the missing DO_NOT_TRACK guard.
