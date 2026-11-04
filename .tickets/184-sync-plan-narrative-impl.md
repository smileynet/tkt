---
id: "184"
title: "Implement sync-plan narrative PLAN.md support (PlanFormat/PlanState) per ADR 0002"
status: backlog
blocked_by: []
priority: medium
validation_criteria:
  - "Narrative support remains deliberately deferred and no parser/format expansion is presented as approved implementation"
  - "Any verified independent PlanState correctness defect is tracked separately from narrative support"
  - "Future activation revalidates the parser/ownership approach against ADR 0002 and the then-current source"
tags: ["planning", "kind:enhancement", "stream:quality", "triage:needs-info"]
---

# Implement sync-plan narrative PLAN.md support (PlanFormat/PlanState) per ADR 0002

## Owner decision — 2026-11-03

Confirmed in chat: **Keep narrative support deferred**. This supersedes conflicting
requirements and the earlier dated decision gate; it does not claim implementation.
See [the authoritative record](../docs/decision-record-2026-11-03.md).

## What to build

Keep optional narrative plan parsing deferred. The primary derived projection direction remains ticket 187 / ADR 0002, and user-owned prose is preserved. No narrative parser implementation or format expansion is activated by this decision. When this ticket is revisited, re-check whether any independent PlanState correctness issue still exists and capture it separately instead of coupling a live repair to deferred parsing.

## Design (from ADR 0002 + #183 research)

Extension point — the drift/finding/gate logic (`sync_plan.rs:62-119`) is already
format-agnostic once fed typed pairs. Introduce:

- `PlanFormat` — a **2-variant enum** (table | task-list), NOT a trait/plugin
  framework (rule-of-two; add a third only when a real third format exists).
  - `extract(plan_text) -> Vec<(TicketId, PlanState)>` (replaces the `RE_PLAN_ROW`
    scan at `sync_plan.rs:42,88`)
  - `rewrite(plan_text, id, desired) -> plan_text` (replaces the inline table fixer
    at `sync_plan.rs:50-60`)
  - dispatch by filename/content sniff; keep `docs/plan.md` table working via
    **auto-detect, not replacement**.
- `PlanState` sum type — `Done | NotDone | Unknown { raw }`, replacing `plan_done:
  bool`. An ASCII-mode or unparseable cell becomes `Unknown` (reportable), never a
  silent not-done. **This is defect 1's structural fix.** The done predicate becomes
  a `PlanFormat` method returning `PlanState`, not a `✅`-substring test.
- `TicketId` new-type over raw `\d+`; the sigil-anchored-ID rule (`#NN` at start/end
  of a list item or heading) is its parser. Parse-don't-validate at the boundary.
- Narrative parse strategy (ADR 0002): **CommonMark AST for structure** (headings =
  phases, list items = tasks; ignore everything else), **narrow anchored regex for
  IDs** over the flattened text of leaf blocks. Never regex for structure. GFM
  task-list checkbox state parsed explicitly (it's an extension, not core CommonMark).
- Drift findings: closed sum type over the five buckets (`TICKET_NOT_IN_PLAN`,
  `PLAN_REF_NO_TICKET`, `STATUS_DRIFT`, `AMBIGUOUS_REF`, `DUPLICATE_REF`), each with
  `file:line` provenance. Warn+suggest on ambiguous IDs; never auto-fix structure.

## Scope boundaries

- `--fix` for narrative is riskier (edit a checkbox/marker, preserve prose). Scope it
  down or disable it for narrative initially until a safe editor exists.
- Dep budget (AGENTS.md: "no deps without justification"): benchmark a hand-rolled
  heading+list-item scanner BEFORE pulling a CommonMark crate (pulldown-cmark/comrak);
  record the decision.
- Position A (plan-as-projection / regenerate from `tkt query`) is explicitly OUT of
  scope — separate follow-up if pursued.

## Acceptance criteria

- [ ] Narrative support remains deliberately deferred and no parser/format expansion is presented as approved implementation
- [ ] Any verified independent PlanState correctness defect is tracked separately from narrative support
- [ ] Future activation revalidates the parser/ownership approach against ADR 0002 and the then-current source

## References

- Governing decision: `docs/adr/0002-plan-narrative-support.md`
- Research: `.scratch/research/prior-art.md`, `.scratch/research/best-practices.md`
- Internal review: `.scratch/internal/syncplan-code.md`, `.scratch/internal/docs-config.md`
- Code: `src/commands/sync_plan.rs`; `core::atomic_write` (already used by the fixer since #183)
- Origin: #183 (decision ticket)

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `planning`.

ADR 0002 governs optional narrative parsing; table projection remains #187. Decide how to isolate the PlanState/ASCII correctness fix from the deferred parser and resolve benchmark-vs-AST wording before building.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Resolved by the owner's 2026-11-03 chat submission: **Keep narrative support deferred**.
See [the decision record](../docs/decision-record-2026-11-03.md). Backlog remains deliberately deferred.

## Cross-project evidence review — 2026-11-03

Crew Research ticket 149 and existing root PLAN.md files establish narrative-planning use, but not a bounded failure that the completed derived projection (#187) cannot serve. The owner deferral and ADR 0002 remain applicable; no parser implementation or dependency is activated from file popularity alone. Independent critic review found no reason to reactivate this ticket. See [the evidence review](../docs/evidence/cross-project-review-2026-11-03.md).
