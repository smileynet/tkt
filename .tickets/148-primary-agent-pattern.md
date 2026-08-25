---
id: "148"
title: "Support primary-agent role pattern via init snippets and documentation"
status: backlog
blocked_by: []
priority: low
validation_criteria:
  - "tkt init --role reviewer generates reviewer-scoped snippet (no claim/close instructions)"
  - "tkt init --role observer generates read-only snippet"
  - "README documents multi-agent role pattern with config examples"
  - "Existing requires/capabilities filtering works unchanged for role tokens"
---

# Support primary-agent role pattern via init snippets and documentation

## What to build

Enable a multi-agent pattern where one agent (e.g. kiro) is the primary executor and others (codex, cursor) are supporting reviewers/observers. Uses the existing `requires`/`machine.capabilities` ABAC mechanism — no new filtering logic needed.

### Deliverable 1: `tkt init --role <executor|reviewer|observer>`

Add a `--role` flag that tailors the deployed snippet content:

- **executor** (default, backward compatible): current full workflow (ready → claim → close)
- **reviewer**: instructs agent to file findings as tickets (`tkt new --requires executor`), run `query`/`validate`, but explicitly says "do not claim or close"
- **observer**: read-only instructions (query, ready --json, blocked, validate)

Works with existing `--target` flag: `tkt init --role reviewer --target claude`

### Deliverable 2: README "Multi-Agent Roles" section

- Pattern explanation: primary declares `executor` capability, execution tickets use `requires: [executor]`
- Example configs for primary vs support machines
- Reviewer → executor handoff protocol via `tkt new --requires executor`

### Not building

- No new frontmatter fields (reuses `requires`)
- No new filtering logic (existing capabilities matching suffices)
- No runtime enforcement at `tkt claim` (advisory via instructions)
- No `assigned_to` field

## Acceptance criteria

- [ ] `tkt init --role reviewer --target agents` produces snippet without claim/close instructions
- [ ] `tkt init --role observer --target agents` produces read-only snippet
- [ ] `tkt init` (no --role) remains backward compatible (executor is default)
- [ ] README has "Multi-Agent Roles" section with config examples
- [ ] Integration test: ticket with `requires: [executor]` hidden from machine without that capability (already works — confirm with test)

## Research

Findings in `.scratch/research/`:
- `agent-role-scoping.md` — TBAC, Linear/Jira models, enforcement patterns
- `capabilities-as-roles.md` — ABAC vs RBAC, why flat capabilities > role hierarchies
- `multi-agent-roles.md` — CIV pattern, PER pipeline, 4-persona system
- `common-agent-roles.md` — industry role taxonomy (5 primary + extended)
- `primary-secondary-collaboration.md` — handoff protocols, Code Mower, ACS
- `ticket-visibility-patterns.md` — Freshdesk scope, exclusion rules, guardrails
- `role-config-deployment.md` — per-agent files, permission scoping, persona patterns
