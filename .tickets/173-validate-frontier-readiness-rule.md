---
id: "173"
title: "validate: frontier-scoped readiness rule (Definition of Ready gate)"
status: open
blocked_by: []
validation_criteria:
  - "Frontier-only readiness criteria are explicit, bounded and advisory with no new implicit hard gate"
  - "Source-reference logic is reused rather than duplicated and its scope is distinct from broader readiness"
  - "Tests preserve frontier membership/order and predictable quiet/structured diagnostics"
tags: ["quality", "kind:enhancement", "stream:guidance", "triage:ready-for-agent"]
---

# validate: frontier-scoped readiness rule (Definition of Ready gate)

## Owner decision — 2026-11-03

Confirmed in chat: **Start with bounded advisory readiness checks**. This supersedes conflicting
requirements and the earlier dated decision gate; it does not claim implementation.
See [the authoritative record](../docs/decision-record-2026-11-03.md).

## Context

Filed from crew-research (companion to its tickets 148 = judgment layer, 149 = plan slim).
`tkt validate` today checks structural correctness (cycles, dangling blocked_by, id↔filename)
and `audit`/`close` check the CLOSE event (unchecked ACs, TBD resolution). **Nothing checks
that an OPEN, frontier-eligible ticket (deps done) has enough to START.** The content bar
lives only in prose (ticket-standards.md), which decays with no mechanical backstop and is
never re-checked against the standing backlog.

This is the MECHANICAL half of a Definition-of-Ready gate; crew-research owns the JUDGMENT
half (is the context actually sufficient). Mechanical proves the slots are filled; judgment
proves the fills are adequate.

**Prior art (directly precedented, not novel):**
- Jira **Field Required Validator** — blocks a workflow transition until required fields are
  present ("Transition failed. Field X is required"). Native. The near-exact analog.
- Linear **LineGuard** / **Required** — webhook-driven: revert the state change + comment what's
  missing. Proves the "warn/revert + explain" UX.

## What to build

Bound frontier readiness to explicit advisory findings, reuse the source-reference logic from ticket 175, and keep source-reference checking separate from broader readiness. Do not introduce a new hard blocker or silently make advisories mandatory under strict mode. Document the exact scope and prove that normal frontier selection/order is unchanged.

## References

- Prior art is summarized inline above (Jira Field Required Validator; Linear LineGuard/Required;
  GitHub Issue Forms; GitLab) — this ticket is self-contained; no external file needed to implement it.
- Companions in the crew-research repo (context only, not required to build this): ticket 148
  (the judgment-layer half) and ticket 149 (PLAN.md slim). Their shared design write-up lives at
  crew-research `docs/agents/specs/ticket-plan-handoff-workstream.md` § Part 1 if that repo is available.

## Acceptance criteria

- [ ] Frontier-only readiness criteria are explicit, bounded and advisory with no new implicit hard gate
- [ ] Source-reference logic is reused rather than duplicated and its scope is distinct from broader readiness
- [ ] Tests preserve frontier membership/order and predictable quiet/structured diagnostics

## Field report — 2026-11-01

Related finding: `validate` doesn't flag an open ticket that can never start because a prerequisite is in backlog (philippines-trip #44 waited on backlog #38 and #43). Proposed advisory: "stuck" — open ticket with a transitive backlog prerequisite.

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `quality`.

Deferred readiness proposal overlaps #175 source-reference rule. Separate frontier readiness from transitive backlog-blocker advisory and decide strict-mode policy before adding duplicate findings.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Resolved by the owner's 2026-11-03 chat submission: **Start with bounded advisory readiness checks**.
See [the decision record](../docs/decision-record-2026-11-03.md). Backlog remains deliberately deferred.
