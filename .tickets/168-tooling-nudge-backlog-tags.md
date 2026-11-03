---
id: "168"
title: "Measure demand for missing-tag and backlog advisories"
status: backlog
blocked_by: ["167"]
priority: low
validation_criteria:
  - "Representative observations state scope, frequency, sources and limits without fabricated or newly enabled telemetry"
  - "The report distinguishes recurring demand from hypothetical benefit and recommends whether an advisory is justified"
  - "No runtime nudge, hard gate or default-status behavior is added by the measurement task"
tags: ["guidance", "kind:enhancement", "stream:guidance", "triage:ready-for-agent"]
---

# Measure demand for missing-tag and backlog advisories

## Owner decision — 2026-11-03

Confirmed in chat: **Measure demand before adding nudges**. This supersedes conflicting
requirements and the earlier dated decision gate; it does not claim implementation.
See [the authoritative record](../docs/decision-record-2026-11-03.md).

## Problem

Guidance alone (ticket #167) may not fully correct under-tagging and over-backlogging,
because the tool currently offers **zero feedback**: `tkt new` succeeds silently with no
tags, and nothing ever flags a growing untagged/backlog population. The code review
(2026-08-29, `.scratch/ticket-behavior-review/review-code.md`) found clean, low-risk hooks
to add a lightweight, non-blocking nudge. This ticket is DEFERRED until #167 (the docs
rebalance) lands and we can measure whether guidance alone moved the numbers.

## What to build

Measure representative missing-tag/backlog workflow cases before proposing runtime nudges. Use scoped primary observations and attributed field reports, preserving telemetry consent and privacy. Record whether a recurring problem exists and whether an opt-in advisory is justified. Do not add a runtime reminder, default-status setting or hard gate as part of this measurement decision.

## Context

- **Hooks identified (from review-code.md / review-config.md):**
  - Creation advisory: success branch of `new::run` (`src/commands/new.rs:137-149`), after
    `effective_tags` computed (`new.rs:72-82`), gated on `!is_quiet()`.
  - Corpus finding: add `check_missing_tags` in `src/findings.rs` (grep for "tag" there =
    0 matches today), register in `collect_findings` (`src/commands/validate.rs:30-38`) and
    doctor (`src/commands/doctor.rs:374-380`).
  - `tags` is absent from lint `CANONICAL_ORDER` (`src/commands/lint.rs:9-17`) — adding it would
    let lint normalize tag placement (optional sub-decision).
  - `new.default_status` would mirror `new.default_priority` (`PROJECT_KEYS` entry + struct field
    + `apply_value` arm + `new.rs` fallback); env override `TKT_NEW_DEFAULT_STATUS` comes free.
- **Design constraint (research-agent-guidance.md):** keep advisories quiet and non-blocking —
  an over-rigid gate is an anti-pattern, and nagging output trains agents to ignore stderr.

## Out of scope

- Any hard enforcement (blocking `new` on missing tags, erroring on backlog). Advisories only.
- A tag allowlist/vocabulary config — separate exploration if desired.

## Acceptance criteria

- [ ] Representative observations state scope, frequency, sources and limits without fabricated or newly enabled telemetry
- [ ] The report distinguishes recurring demand from hypothetical benefit and recommends whether an advisory is justified
- [ ] No runtime nudge, hard gate or default-status behavior is added by the measurement task

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `guidance`.

Deferred advisory proposal after completed #167. First measure whether nudges are still wanted; keep quiet/JSON behavior and do not promote advisory tag checks into a mandatory gate.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Resolved by the owner's 2026-11-03 chat submission: **Measure demand before adding nudges**.
See [the decision record](../docs/decision-record-2026-11-03.md). Backlog remains deliberately deferred.
