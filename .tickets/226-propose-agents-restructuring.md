---
id: "226"
title: "Propose AGENTS.md restructuring while preserving guidance coverage"
status: open
blocked_by: []
validation_criteria:
  - "Proposal maps every moved section to an existing or new reference, preserves operational and safety rules, and updates the guidance-surface matrix coherently"
tags: ["guidance"]
---

# Propose AGENTS.md restructuring while preserving guidance coverage

## What to build

Prepare a reviewable proposal for reducing always-loaded reference material in AGENTS.md without weakening the project's operational instructions or deliberate guidance coverage. Identify exact sections to retain, extract, or replace with conditional pointers. Do not implement the restructuring before proposal approval.

## Context

Intent source: the 2026-10-29 conformance audit, independent critic review, and the user's request to track this follow-up.

- AGENTS.md currently includes CLI flags, configuration, architecture decisions, operational commands, and safety constraints.
- docs/agents/agent-guidance-surfaces.md explicitly assigns CLI/config coverage to AGENTS.md. Length alone is not a defect; any proposed extraction must update that coverage contract coherently.
- Apply grey-matter's writing-for-agents progressive-disclosure guidance as a design lens, while preserving this repository's documented requirements.
- Existing references include skills/tkt-core/references/commands.md, docs/agents/specs/workspace-layout.md, docs/agents/session-operations.md, and docs/adr/.

## Acceptance criteria

- [ ] Inventory sections and justify what stays always loaded versus what moves behind a pointer.
- [ ] Map every proposed extraction to an existing or proposed destination and specify when its pointer should trigger.
- [ ] Preserve verification, commit/push/deploy, standalone-skill, release authorization, and platform safety requirements.
- [ ] Propose matching changes to the guidance-surface matrix and update checklist; avoid duplicating authoritative reference content.
- [ ] Present the proposal for approval before implementing AGENTS.md restructuring.

## Out of scope

CLI behavior changes, release/version changes, broad README rewrites, and implementing the proposed restructuring without approval.
