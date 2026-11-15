---
id: "292"
title: "Make advisory audit outcomes explicit and reconcile superseded acceptance"
status: done
blocked_by: []
priority: medium
validation_criteria:
  - "cargo test audit covers clean, warning, and strict outcomes"
  - "audit JSON adds counts and pass_with_warnings without changing status or exit compatibility"
  - "tickets 46 and 121 record explicit decisions and mise run check passes"
tags: ["stream:quality", "kind:enhancement", "triage:ready-for-agent"]
assignee: "kimi"
---
# Make advisory audit outcomes explicit and reconcile superseded acceptance

## Intent source

Owner-approved audit on 2026-11-13 and implementation approval on 2026-11-15. Current `tkt audit --deep` emits JSON `status: pass` and exit zero with eighteen warnings. Ticket 121 retains a conflicting warning-exit acceptance criterion; ticket 46 checks research that its resolution explicitly deferred.

## Context

- `src/findings.rs` owns shared finding status/output behavior.
- `src/commands/audit.rs` and `src/audit.rs` own audit execution/rules.
- `tests/integration.rs` intentionally pins advisory exit zero.
- Ticket 34 describes audit as a soft quality signal; ticket 121 contains the conflicting exit-code criterion.
- `docs/evidence/historical-closure-239.md` already distinguishes advisory findings from clean evidence.
- Tickets 46 and 121 are historical records: do not invent contemporaneous evidence.

## What to build

Audit output makes warning-bearing advisory results unmistakable while preserving exit codes and existing `status` compatibility. Add structured warning/error/info counts and an additive outcome such as `pass_with_warnings`; human brief output says `pass with warnings` when appropriate. Apply equivalent additive clarity to validate only if it can share the existing output path without changing its contract.

Record an explicit superseding decision for ticket 121 and a decision/follow-up disposition for ticket 46's deferred research. The new priority-frontier work owns any resulting implementation demand; no historical AC or proof is rewritten.

## Acceptance criteria

- [x] Warning-only audit JSON retains `status: pass`, adds counts, and exposes `outcome: pass_with_warnings`.
- [x] Warning-only audit brief output says `pass with warnings` and retains exit zero.
- [x] Clean and strict-failure outcomes remain compatible and have regression tests.
- [x] Ticket 121 records the current advisory contract as an explicit supersession of its old exit-code AC.
- [x] Ticket 46 records whether the deferred usage research remains demand or is rejected, without claiming it was historically performed.
- [x] The full maintainer gate passes.

## Out of scope

Promoting warnings to failure by default, semantic proof validation, editing historical checked boxes, or suppressing unresolved historical findings.

## Resolution (2026-11-15)

Added audit outcome and severity counts while preserving status and exit compatibility, documented the contract, and recorded historical supersession/disposition without rewriting acceptance or proof.

### Verification
1. ✓ cargo test audit covers clean, warning, and strict outcomes — "cargo test --test audit_outcomes => 3 passed for clean, warning-only and strict; audit_template_content => 1 passed"
2. ✓ audit JSON adds counts and pass_with_warnings without changing status or exit compatibility — "Live deep audit: status pass, outcome pass_with_warnings, counts errors=0 warnings=18 info=0; brief says pass with warnings; strict regression retains fail/exit 1"
3. ✓ tickets 46 and 121 record explicit decisions and mise run check passes — "Notes added to 121 and 46 explicitly supersede warning-exit AC and reject retroactive research proof while identifying #294 demand; mise run check completed successfully"
