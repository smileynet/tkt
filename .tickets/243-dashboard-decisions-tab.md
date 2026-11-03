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

- [ ] Decisions tab contains current decision gates with clear options, a default recommended radio choice, and optional free text for each.
- [ ] Bottom copy button exports selected choices and notes; clipboard denial produces selectable fallback text.
- [ ] Canonical template/example, schema checker and deterministic interaction/format tests pass; all prior entries and tickets survive migration.
- [ ] Actual laptop/phone layout and interactions are inspected when a permitted browser is available.

## Planning gate

Local browser use remains disabled. Model/DOM-simulation checks can be performed; rendered acceptance remains unverified and no dashboard feedback/review request is made to the owner.
