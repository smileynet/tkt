---
id: "88"
title: "tkt done --week: show-resolved report for standups"
status: done
blocked_by: []
priority: low
validation_criteria:
  - "Resolved-work dates come from a verified done transition in Git history"
  - "Missing/shallow/ambiguous history is explicitly unknown without a fabricated date or new completion field"
  - "Window/timezone and cancelled-versus-done behavior are documented and tested"
tags: ["cli", "kind:enhancement", "stream:workflow", "triage:ready-for-agent"]
assignee: "codex"
---

# tkt done --week: show-resolved report for standups

## Owner decision — 2026-11-03

Confirmed in chat: **Use Git closure history; mark unknown dates**. This supersedes conflicting
requirements and the earlier dated decision gate; it does not claim implementation.
See [the authoritative record](../docs/decision-record-2026-11-03.md).

## Context

dstask's show-resolved weekly view is useful for standups and retrospectives. "What did I finish this week?" is a common question tkt should answer without manual git log digging.

## What to build

Use verified Git closure history as the completion-time source. When history is absent, shallow, ambiguous or cannot establish identity after renumbering, report an unknown date rather than inventing a timestamp or silently using an unrelated date. Define the reporting window/timezone explicitly and distinguish cancellation from completed work.

## Acceptance criteria

- [x] Resolved-work dates come from a verified done transition in Git history
- [x] Missing/shallow/ambiguous history is explicitly unknown without a fabricated date or new completion field
- [x] Window/timezone and cancelled-versus-done behavior are documented and tested

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `cli`.

Deferred reporting feature. Decide authoritative completion timestamp and behavior when git history is unavailable; mtime is not proof of completion time.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Resolved by the owner's 2026-11-03 chat submission: **Use Git closure history; mark unknown dates**.
See [the decision record](../docs/decision-record-2026-11-03.md). Backlog remains deliberately deferred.


## Reactivation — 2026-11-03

The owner requested review of all parked work and execution of actionable tickets. This supersedes earlier blanket deferral. See [the parked-work review](../docs/parked-review-2026-11-03.md) for current scope and sequencing. Independent critic review is required before closure.

## Resolution (2026-11-04)

Implemented current-done reports with bounded same-path/same-ID Git transition attribution, inclusive rolling UTC week, explicit unknown dates and separate dated/unknown counts. No completion field added. Independent critic accepted final source, guidance and passing gate; installed CLI freshness passed at c15919f.

### Verification
1. ✓ Resolved-work dates come from a verified done transition in Git history — "Raw committer transition fixtures prove later edits, body dates,mtime and author time do not move completion dates; latest reclosure selected. Public report suite11/11 and full Rust482 pass."
2. ✓ Missing/shallow/ambiguous history is explicitly unknown without a fabricated date or new completion field — "Root,missing,uncommitted,duplicate,rename/ID discontinuity,merge,shallow,unparseable,missing promisor and resource limits return unknowns; whole-repository snapshots unchanged; no lazy fetch positive oracle. See docs/evidence/done-report-88.md."
3. ✓ Window/timezone and cancelled-versus-done behavior are documented and tested — "Inclusive 604800-second UTC boundary unit test plus cancelled/reopened exclusion and rolling-week integration tests pass; documentation explains unknown membership and history limitations. Gate42 Node pass/1 existing Windows skip,fmt clean,clippy zero warnings; critic accepted before close."
