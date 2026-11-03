---
id: "230"
title: "Separate artifact, functionality and user-outcome evidence in ticket closure guidance"
status: open
blocked_by: []
validation_criteria:
  - "tkt-work documents classified evidence with observable check output and unavailable checks"
  - "Skill validation and applicable guidance tests pass without adding non-tkt skill dependencies"
tags: ["guidance", "kind:enhancement", "triage:ready-for-agent", "stream:guidance"]
---

# Separate artifact, functionality and user-outcome evidence in ticket closure guidance

## What to build

Intent: user-approved recommendations from Learning Library ticket 13 retrospective (31 October 2026). Its static reader had passing artifact checks, permitted HTTP preview observations, blocked tool file-navigation and a separate human direct-file report. None establishes learning effectiveness.

Read skills/tkt-work/SKILL.md, docs/agents/agent-guidance-surfaces.md and existing evidence-gate guidance. Clarify closure evidence as check command plus observed result and classification; distinguish artifact correctness, tool-observed behavior, human-reported functionality and user outcomes. Unavailable checks stay unavailable. Keep this portable: no dependency on grey-matter dashboard or other non-tkt skills. Project-specific dashboard post-close refresh remains a project responsibility.

## Acceptance criteria

- [ ] tkt-work explains command/result evidence and explicit failure, skip and infrastructure limitation classifications.
- [ ] Examples keep human-reported functionality separate from observed execution and user outcomes.
- [ ] Applicable guidance fixtures/checks pass; no non-tkt skill dependency is introduced.

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `guidance`.

Own classified artifact/functionality/user-outcome evidence examples, including the evidence refinement previously appended to #177. No duplicate implementation in #177.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.
