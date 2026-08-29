---
id: "168"
title: "Optional tooling nudge: advisory on tag-less/backlog tickets"
status: backlog
blocked_by: ["167"]
priority: low
validation_criteria:
  - "tkt new prints a non-blocking advisory to stderr when a ticket is created with no tags (gated on !quiet)"
  - "validate or doctor emits an advisory (not error) counting untagged live tickets, via a check_missing_tags finding in findings.rs"
  - "decision recorded on whether to add a new.default_status config key (mirroring new.default_priority) or keep the hard-coded open default"
---

# Optional tooling nudge: advisory on tag-less/backlog tickets

## Problem

Guidance alone (ticket #167) may not fully correct under-tagging and over-backlogging,
because the tool currently offers **zero feedback**: `tkt new` succeeds silently with no
tags, and nothing ever flags a growing untagged/backlog population. The code review
(2026-08-29, `.scratch/ticket-behavior-review/review-code.md`) found clean, low-risk hooks
to add a lightweight, non-blocking nudge. This ticket is DEFERRED until #167 (the docs
rebalance) lands and we can measure whether guidance alone moved the numbers.

## What to build

A minimal, non-blocking tooling nudge that reinforces the guidance without adding friction:

- **Creation advisory** — when `tkt new` creates a ticket with no tags (and not `-q`), print
  a one-line advisory to stderr suggesting `--tags`. Never blocks, never fails.
- **Corpus advisory** — a `check_missing_tags` finding (advisory/warning, never error) that
  reports the count/percentage of untagged live tickets, surfaced via `validate` and/or
  `doctor`.
- **Config decision** — decide whether to add a `new.default_status` config key (mirroring
  the existing `new.default_priority` pattern at `src/config.rs` + `src/commands/new.rs`) so a
  project can pin the creation default, or keep the hard-coded `open` fallback
  (`src/core/ticket.rs:947`). Record the decision (ADR or ticket note).

## Context

- **Hooks identified (from review-code.md / review-config.md):**
  - Creation advisory: success branch of `new::run` (`src/commands/new.rs:137-149`), after
    `effective_tags` computed (`new.rs:72-82`), gated on `!is_quiet()`.
  - Corpus finding: add `check_missing_tags` in `src/core/findings.rs` (grep for "tag" there =
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

- [ ] `tkt new` prints a non-blocking stderr advisory when a ticket is created with no tags,
      suppressed under `-q`
- [ ] `validate` or `doctor` emits an advisory (not error) counting untagged live tickets, via
      a `check_missing_tags` finding in `findings.rs`
- [ ] Decision recorded on `new.default_status` config key vs keeping the hard-coded `open` default
- [ ] `cargo fmt && cargo clippy --all-targets && cargo test` all pass with zero warnings
