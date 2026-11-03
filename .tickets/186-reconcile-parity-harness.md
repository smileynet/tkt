---
id: "186"
title: "Reconcile or retire the historical Python parity harness"
status: done
blocked_by: []
priority: medium
validation_criteria:
  - "Unsupported parity runner/instructions are retired without inventing a supported Python reference"
  - "Useful contract cases are preserved in maintained Rust tests where needed and actual current coverage is named"
  - "References and verification docs no longer imply that the dormant Python comparison is a passing gate"
tags: ["quality", "tests", "kind:bug", "stream:quality", "triage:ready-for-agent"]
assignee: "codex"
---

# Reconcile or retire the historical Python parity harness

## Evidence correction — 2026-11-03

The three documented fixture files were present at review time; the old claim that they were absent was incorrect. They remain preserved and are now consumed by the maintained Rust test test_historical_fixture_contract. The unsupported Python comparison runner is retired; no supported Python reference exists in this checkout and no historical comparison is claimed to pass. Current confidence comes from the Rust gate, not implementation parity. This observation supersedes conflicting historical context below.

## Owner decision — 2026-11-03

Confirmed in chat: **Retire the unsupported harness**. This supersedes conflicting
requirements and the earlier dated decision gate; it does not claim implementation.
See [the authoritative record](../docs/decision-record-2026-11-03.md).

## Context

`tests/parity/README.md` describes a Rust-versus-Python comparison harness, but this
repository no longer contains the Python implementation or the documented fixtures.
The harness is not part of the passing Rust test suite, so it must not be represented
as current release confidence without a runnable reference.

## What to build

Retire the unsupported historical Python parity harness and obsolete runnable instructions. Preserve any useful contract cases in maintained Rust tests and name the actual unit/integration verification path. Do not claim that the absent Python reference or old parity run passed. This approves retirement work; the ticket is not complete until the source and documentation changes are verified.

## Acceptance criteria

- [x] Unsupported parity runner/instructions are retired without inventing a supported Python reference
- [x] Useful contract cases are preserved in maintained Rust tests where needed and actual current coverage is named
- [x] References and verification docs no longer imply that the dormant Python comparison is a passing gate

## Re-triage (2026-10-29)

`tests/parity/` still holds `README.md`, `run.sh` and `fixtures/` at main `980b043`.
First check whether `run.sh` still runs against anything; then either retire the harness
or document what it does today.

## Triage — 2026-11-02

Disposition: `kind:bug`, `triage:needs-info`; stream `quality`.

tests/parity/run.sh still expects python -m tkt (or an external command), absent from tracked source. No runnable Python reference was found; choose retirement or provide a pinned reference, never count the dormant harness as a passing check.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Resolved by the owner's 2026-11-03 chat submission: **Retire the unsupported harness**.
See [the decision record](../docs/decision-record-2026-11-03.md). Backlog remains deliberately deferred.


## Reactivation — 2026-11-03

The owner requested review of all parked work and execution of actionable tickets. This supersedes earlier blanket deferral. See [the parked-work review](../docs/parked-review-2026-11-03.md) for current scope and sequencing. Independent critic review is required before closure.

## Resolution (2026-11-03)

Retired unsupported Python comparison runner, retained all three fixtures under maintained Rust regression, corrected historical fixture claim; critic accepted.

### Verification
1. ✓ Unsupported parity runner/instructions are retired without inventing a supported Python reference — "The unsupported runner was recycled. Active instructions describe maintained Rust coverage and make no Python parity claim."
2. ✓ Useful contract cases are preserved in maintained Rust tests where needed and actual current coverage is named — "Three fixtures remain in a native regression; the full gate passed 402 Rust tests and 42 Node fixtures, with one existing Windows skip."
3. ✓ References and verification docs no longer imply that the dormant Python comparison is a passing gate — "References and diff checks passed. The incorrect historical fixture-absence claim was corrected. Independent critic review accepted retirement. See docs/evidence/parity-retirement-186.md."
