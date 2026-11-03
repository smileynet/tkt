---
id: "187"
title: "Generate a derived sync-plan projection from the ticket corpus"
status: done
blocked_by: []
priority: medium
validation_criteria:
  - "A projection command renders every relevant ticket from the corpus with deterministic ordering and derived status"
  - "Generated output is written atomically and respects global dry-run"
  - "The command preserves or clearly separates any non-derivable human narrative"
tags: ["planning", "sync-plan", "kind:enhancement", "triage:ready-for-agent", "stream:quality"]
assignee: "codex"
---

# Generate a derived sync-plan projection from the ticket corpus

## Context

ADR 0002 makes plan-as-projection the strategic primary direction for `sync-plan`:
ticket files remain the source of truth, while derivable plan content is rendered from
the corpus. #184 deliberately implements the secondary, opt-in narrative format and
explicitly excludes this work. Without a dedicated ticket, the ADR's primary direction
has no owner or schedulable scope.

## What to build

Design and implement a command or safe `sync-plan` mode that renders a deterministic
table projection from the ticket corpus. Define where generated content lives and how
human-authored, non-derivable narrative is preserved or separated. The generated path
must be safe to preview, atomic to write, and must not make a hand-maintained plan a
second source of truth.

This is intentionally separate from #184: projection is the primary derived-plan path;
#184 is optional parsing of a human-authored narrative plan.

## Acceptance criteria

- [x] A projection command/mode renders all relevant tickets in deterministic priority/ID order with derived status
- [x] The generated output is atomically written and global `--dry-run` previews without writing
- [x] The location and behavior of human-authored narrative are explicitly preserved or separated
- [x] Table-format `sync-plan` compatibility and #184's opt-in narrative design remain intact
- [x] Help, README, and agent guidance explain ticket files as the source of truth

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `planning`.

Deferred deterministic projection per ADR 0002; current sync-plan only checks/fixes tables. Preserve human narrative and independent optional #184 scope.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.


## Reactivation — 2026-11-03

The owner requested review of all parked work and execution of actionable tickets. This supersedes earlier blanket deferral. See [the parked-work review](../docs/parked-review-2026-11-03.md) for current scope and sequencing. Independent critic review is required before closure.

## Resolution (2026-11-03)

Implemented deterministic owned-region derived projection, safe publication and source/ownership guards; fixed triggered capabilities debug-stack regression; final full gate and critic accepted.

### Verification
1. ✓ A projection command renders every relevant ticket from the corpus with deterministic ordering and derived status — "Whole-corpus ordering, all statuses, slug IDs, context independence and table/JSON compatibility have public coverage. The full gate passed 419 Rust tests and 42 Node fixtures, with one existing Windows skip."
2. ✓ Generated output is written atomically and respects global dry-run — "Dry-run preserves absent/existing output. New-file publication refuses a foreign arrival; stale existing snapshots are rejected and locked output stays intact. Remaining single-writer and filesystem limits are documented."
3. ✓ The command preserves or clearly separates any non-derivable human narrative — "Only the uniquely owned, unfenced region changes; human prefix/suffix bytes and CRLF survive. Independent critic review accepted ownership and the capabilities stack fix with its 32-process regression. See docs/evidence/plan-projection-187.md."
