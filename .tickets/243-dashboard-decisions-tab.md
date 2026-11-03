---
id: "243"
title: "Collect dashboard decisions with recommendations and chat export"
status: in_progress
blocked_by: []
validation_criteria:
  - "Recommended radio choices and notes export deterministically with clipboard fallback"
  - "Canonical checks and complete snapshot preservation pass; unavailable rendered checks remain explicit"
tags: ["dashboard", "kind:enhancement", "triage:ready-for-agent", "stream:quality", "gate:evidence"]
assignee: "codex"
---
# Dashboard decisions tab

## Intent source

Owner request 2026-11-03: show decisions in their own tab; each has multiple-choice radio options with the recommendation selected, an open notes box, and a button at the bottom to copy all answers into chat.

## What to build

Extend the canonical delivery-dashboard renderer and checker, then adopt it without losing project entries, config or ticket history. Curate every current decision gate, distinguish deferred decisions, and include the pending guidance proposal. Use accessible fieldsets, labels, keyboard navigation and safe plain-text export. Clipboard failure must leave selectable text. Copying is a draft handoff; no ticket or repository changes occur from the page.

## Acceptance criteria

- [x] Decisions tab contains current decision gates with clear options, a default recommended radio choice, and optional free text for each.
- [x] Bottom copy button exports selected choices and notes; clipboard denial produces selectable fallback text.
- [x] Canonical template/example, schema checker and deterministic interaction/format tests pass; all prior entries and tickets survive migration.
- [ ] Actual laptop/phone layout and interactions are inspected when a permitted browser is available.

## Planning gate

Local browser use remains disabled. Model/DOM-simulation checks can be performed; rendered acceptance remains unverified and no dashboard feedback/review request is made to the owner.

## Implementation evidence — 2026-11-03

The Decisions tab contains 17 curated choices: all 16 recorded decision gates
plus proposal approval for 226. Recommendations are selected by default; notes,
alternative choices and one complete chat export are wired. Clipboard failure
leaves selectable text. Source is aligned to published canonical `a812670`.

Seven actual-renderer DOM-simulation tests and 42 canonical contract tests pass;
the dashboard checker reports zero errors/warnings. All 52 earlier entries and
the prior config are preserved, with a refreshed 243-ticket snapshot.
See [durable evidence](../docs/evidence/dashboard-decisions-243.md).

A fresh computer-use inventory confirms `Local browser use is disabled in Work
settings`. No browser or native-browser fallback was attempted. Rendered
laptop/phone checks and native clipboard behavior remain unverified; keep this
ticket in progress without asking the owner to perform dashboard review.
