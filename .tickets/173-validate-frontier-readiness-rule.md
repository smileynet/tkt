---
id: "173"
title: "validate: frontier-scoped readiness rule (Definition of Ready gate)"
status: done
blocked_by: ["175"]
validation_criteria:
  - "Frontier-only readiness criteria are explicit, bounded and advisory with no new implicit hard gate"
  - "Source-reference logic is reused rather than duplicated and its scope is distinct from broader readiness"
  - "Tests preserve frontier membership/order and predictable quiet/structured diagnostics"
tags: ["quality", "kind:enhancement", "stream:guidance", "triage:ready-for-agent"]
assignee: "codex"
---

# validate: frontier-scoped readiness rule (Definition of Ready gate)

## Bounded implementation scope — 2026-11-03

Audit the project structural frontier using existing dependency/container/capability selection and shared ready capability resolution. The audit is project-wide, not narrowed by invocation identity, ownership, tags or parent-view filters. Info-only missing source/AC/validation slots never change readiness or strict exit status. Reuse the source predicate; source-specific policy remains separately controlled by ticket175. Fenced or moved-only acceptance examples do not fill the AC slot. Transitive backlog-chain diagnosis is outside this first rule and remains future scope, not a claimed result.

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

- [x] Frontier-only readiness criteria are explicit, bounded and advisory with no new implicit hard gate
- [x] Source-reference logic is reused rather than duplicated and its scope is distinct from broader readiness
- [x] Tests preserve frontier membership/order and predictable quiet/structured diagnostics

## Field report — 2026-11-01

Related finding: `validate` doesn't flag an open ticket that can never start because a prerequisite is in backlog (philippines-trip #44 waited on backlog #38 and #43). Proposed advisory: "stuck" — open ticket with a transitive backlog prerequisite.

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `quality`.

Deferred readiness proposal overlaps #175 source-reference rule. Separate frontier readiness from transitive backlog-blocker advisory and decide strict-mode policy before adding duplicate findings.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Resolved by the owner's 2026-11-03 chat submission: **Start with bounded advisory readiness checks**.
See [the decision record](../docs/decision-record-2026-11-03.md). Backlog remains deliberately deferred.


## Reactivation — 2026-11-03

The owner requested review of all parked work and execution of actionable tickets. This supersedes earlier blanket deferral. See [the parked-work review](../docs/parked-review-2026-11-03.md) for current scope and sequencing. Independent critic review is required before closure.

## Resolution (2026-11-03)

Implemented bounded info-only project frontier slot checks, shared native capabilities and source predicate; repaired indirect validate-fix info gate, final full gate and critic accepted.

### Verification
1. ✓ Frontier-only readiness criteria are explicit, bounded and advisory with no new implicit hard gate — "Source, acceptance and validation slots on the project structural frontier are info-only, including under strict mode. The full gate passed 405 Rust tests and 42 Node fixtures, with one existing Windows skip."
2. ✓ Source-reference logic is reused rather than duplicated and its scope is distinct from broader readiness — "The rule reuses the shared source predicate and native frontier/capability selection. Fenced, moved and empty checkbox examples do not fill the acceptance slot; content adequacy and backlog-chain diagnosis are not claimed."
3. ✓ Tests preserve frontier membership/order and predictable quiet/structured diagnostics — "Public tests preserve ready JSON, cover scope and strict/quiet output, and retain warning/error repair regression gates. Independent critic review accepted the source and indirect-gate correction. See docs/evidence/frontier-readiness-173.md."
