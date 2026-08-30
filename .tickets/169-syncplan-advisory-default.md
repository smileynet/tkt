---
id: "169"
title: "sync-plan: advisory-by-default (demote plan-status-drift to warning, repurpose --check as CI gate)"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "default sync-plan run with drift exits 0 (test: integration::sync_plan_advisory_default)"
  - "sync-plan --check exits 1 on drift (test: integration::sync_plan_check_gate)"
  - "sync-plan --fix --dry-run previews without writing (test: integration::sync_plan_fix_dryrun)"
---

# sync-plan: advisory-by-default (demote plan-status-drift to warning, repurpose --check as CI gate)

## Context

Telemetry review (2026-08-30, 1251 events): `sync-plan` returns exit=1 on 27/31 runs (87%). The command behaves as a hard gate against a hand-maintained plan table.

## Root cause (verified in source)

- `sync_plan.rs:64-71` — `plan-status-drift` findings are hardcoded `severity: "error"`.
- `sync_plan.rs:99-105` — exit=1 iff ≥1 error-severity finding OR (`--strict` AND ≥1 warning).
- `sync_plan.rs:22` — `--check` is a **dead flag**: `let _ = check;`. The only real mode axis is fix/no-fix.
- `sync_plan.rs:26` — default plan path is `docs/plan.md` (not `PLAN.md` — docs disagree, see #below).
- `sync_plan.rs:46-60,93-95` — `--fix` regenerates the drifted status cell in-place; proves the drift class is fully **derivable/cosmetic**. No `--dry-run` (unlike `validate`).

Because the plan table lags ticket state constantly and every mismatch is error-grade, nearly every run fails. The dominant exit=1 driver is exactly the derivable class `--fix` can silently repair.

## Prior art (research: .scratch/research/plan-drift.md)

Binary drift detection rots into ignored noise; actionable tools grade drift into tiers and route by severity (silent-fix / warn / block). Governing rule (DevSecOps quality gate): **block only on unacceptable risk.** Terraform-style drift UX filters known-benign classes before comparing. Derivable drift = advisory, not blocking.

## What to build

1. Demote `plan-status-drift` to `severity: "warning"` (advisory) — default run exits 0.
2. Repurpose the dead `--check` flag as the CI gate: `--check` reports drift AND fails on it (exit 1). Keep `--strict` as the escalation knob (warnings→errors).
3. Reserve error/exit=1 for non-derivable conflicts (e.g. a plan row citing an id no ticket has), not cosmetic ✅ mismatches.
4. Add `--fix --dry-run` parity with `validate` — preview without writing `docs/plan.md`.
5. Fix the doc path contradiction: code uses `docs/plan.md`; README + frontier-work steering say `PLAN.md`. Reconcile (docs are wrong) across guidance surfaces per `.memory/agent-guidance-surfaces.md`.
6. Document sync-plan's advisory status + `--check`/`--strict` behavior (currently undocumented on every surface).

## Acceptance criteria

- [x] `plan-status-drift` is `warning`; default `sync-plan` with drift exits 0
- [x] `sync-plan --check` exits 1 when drift exists (repurposed from dead flag)
- [x] `sync-plan --fix --dry-run` previews changes without writing the plan file
- [x] Non-derivable conflicts still error (id in plan with no ticket)
- [x] Docs reconciled: advisory status documented, `docs/plan.md` vs `PLAN.md` fixed across surfaces
- [x] `cargo fmt && cargo clippy --all-targets && cargo test` clean

## Resolution (2026-08-30)

sync-plan now advisory by default (plan-status-drift demoted to warning, exit 0); --check repurposed as CI gate (exit 1 on drift); new plan-orphan-row errors on non-derivable conflicts; --fix honors global --dry-run. Docs reconciled across AGENTS.md/README/SKILL.md/commands.md/frontier-work.md incl. docs/plan.md vs PLAN.md path fix.

### Verification
1. ✓ default sync-plan run with drift exits 0 (test: integration::sync_plan_advisory_default) — "cargo test: 66 passed 0 failed incl sync_plan_advisory_default/check_gate/orphan_row_errors/fix_dryrun"
2. ✓ sync-plan --check exits 1 on drift (test: integration::sync_plan_check_gate) — "e2e on installed binary 3599b04: default exit 0 (advisory), --check exit 1 (gate), --fix --dry-run exit 0 with plan unchanged"
3. ✓ sync-plan --fix --dry-run previews without writing (test: integration::sync_plan_fix_dryrun) — "clippy --all-targets clean; rustfmt --check clean across src+tests"
