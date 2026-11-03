---
id: "159"
title: "Document CI recipe: tkt validate --strict / lint --check as the real enforcement gate"
status: open
blocked_by: []
priority: low
validation_criteria:
  - "README/AGENTS carry a CI snippet running tkt validate --strict and tkt lint --check"
  - "docs state client-side hooks are advisory; CI is the only unbypassable boundary for hand-editable .tickets/"
tags: ["docs", "guidance", "kind:enhancement", "triage:needs-info", "stream:guidance", "gate:decision"]
---

# Document CI recipe: tkt validate --strict / lint --check as the real enforcement gate

## What to build

Document that CI is the only unbypassable enforcement boundary for tkt, and provide a copy-paste recipe. Rationale (from #155 research): `.tickets/*.md` are hand-editable files, so every client-side control (hooks, agent deny-rules) is advisory — an actor can always edit the file directly and `git commit --no-verify`. The real gate is CI running `tkt validate --strict` and `tkt lint --check`.

Add to README and/or AGENTS.md:
- A CI snippet (GitHub Actions example) running `tkt validate --strict` and `tkt lint --check` on PRs, failing the build on contract violations / lint drift.
- A short "enforcement model" note: client hooks warn (fast, local, bypassable); CI `--strict` blocks (the real gate).

## Context

- **Relevant files:** `README.md`, `AGENTS.md`, possibly `skills/tkt-core/references/commands.md`
- Research: `.scratch/subagent-raw/guardrail-warn-vs-block.md` (open question 2 — confirm projects actually wire CI, or blocks are advisory-only in practice)
- Complements #157/#158 (which are the warn-layer); this documents where the block-layer actually lives.

## Acceptance criteria

- [ ] README/AGENTS carry a CI snippet running `tkt validate --strict` and `tkt lint --check`
- [ ] docs state plainly that client-side hooks are advisory and CI is the only unbypassable boundary for hand-editable `.tickets/`
- [ ] the enforcement model (warn local, block CI) is stated in one place, referenced elsewhere

## Out of scope

- Implementing hooks (#157/#158)

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `guidance`.

Old automatic-PR-CI prescription conflicts with current standing manual-dispatch-only verification rule. Propose optional consumer enforcement and accurate bypass limits; do not add hosted CI or describe it as universally unbypassable.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Choose a scoped optional-consumer CI recipe consistent with manual-only verification policy.
