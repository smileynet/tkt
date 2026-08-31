---
id: "173"
title: "validate: frontier-scoped readiness rule (Definition of Ready gate)"
status: open
blocked_by: []
validation_criteria:
  - "open+deps-done ticket missing an intent source (spec/blocked_by/etc) is flagged"
  - "frontier ticket missing a Context section with >=1 path/link is flagged"
  - "frontier ticket with no acceptance-criteria checkbox is flagged"
  - "rule is warn-by-default, error under --strict; re-checks standing backlog not just creation"
---

# validate: frontier-scoped readiness rule (Definition of Ready gate)

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

A new `tkt validate` rule, **frontier-scoped** (only tickets with `status: open` AND all
`blocked_by` done — the set about to be worked). Flag a ticket that lacks:
- an **intent source** — a `spec:` field, a `blocked_by`, or a body section citing the origin
  (user-request / discovery / ADR)
- a **Context** (or equivalent "What to build") section containing >=1 file path or link
- at least one **acceptance-criteria checkbox** (`- [ ]`)

Behavior: **warn by default, error under `--strict`** (matches the existing severity model).
Re-checks the standing backlog on every run, not just at creation (creation-only checks rot).
Emit under the existing JSON finding schema with a new rule name (e.g. `frontier-not-ready`).
Keep it advisory — readiness is a judgment call the tool can only partially prove; do NOT
hard-block work (false positives create friction; see the project's own enforcement-hierarchy note).

## References

- crew-research `.scratch/subagent-raw/ticket-completeness.md` (readiness definition, mechanical vs judgment split)
- crew-research `.scratch/research/dor-tooling-priorart.md` (Jira/Linear/GitLab DoR enforcement survey)
- crew-research tickets 148 (judgment layer), 149 (plan slim) — this is their mechanical companion

## Acceptance criteria

- [ ] `validate` flags a frontier ticket (open + deps done) missing an intent source
- [ ] flags a frontier ticket missing a Context/what-to-build section with >=1 path or link
- [ ] flags a frontier ticket with zero acceptance-criteria checkboxes
- [ ] warn-by-default, error under `--strict`; emitted in the JSON finding schema with a named rule
- [ ] non-frontier tickets (backlog, blocked, done) are NOT flagged by this rule
- [ ] tests cover: ready ticket passes; each missing-field case is flagged; --strict promotes to error
