---
id: "191"
title: "Centralize command mutation effects and dry-run policy"
status: done
blocked_by: ["136"]
priority: medium
validation_criteria:
  - "Every mutating command uses one plan-and-apply effect path"
  - "Dry run performs no persistent filesystem or Git mutation"
  - "The shared executor preserves atomic writes and push gating"
  - "Table-driven tests cover no-write dry-run behavior"
tags: ["architecture", "integrity", "kind:enhancement", "triage:ready-for-agent", "stream:quality"]
assignee: "codex"
---

# Centralize command mutation effects and dry-run policy

## What to build

The global `--dry-run` repair in #136 must land first. Afterwards, consolidate the remaining
command-specific write, atomic-file, Git transaction, and publish decisions behind one small
concrete plan-and-apply executor. Today mutation effects are distributed across commands;
that makes it easy for a new command to bypass dry-run, atomic writes, or push gating (the
migration reviewed-marker is an example to audit).

The goal is not a generic trait/plugin framework and not a merger of `GitTransaction` with
`MutationContext`. Define one testable internal effect plan with preview and apply modes, then
route each mutating command through it while preserving the existing command contracts.

## Acceptance criteria

- [x] Every mutating command enumerates filesystem, Git, and publish effects through one plan-and-apply path
- [x] Dry run performs no persistent filesystem write, Git mutation, marker creation, commit, or push
- [x] The executor preserves `atomic_write` where appropriate and respects `push.enabled`
- [x] Existing command output and error contracts remain compatible
- [x] Table-driven tests cover every mutator's no-write dry-run behavior

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `architecture`.

Deferred consolidation after completed #136. MutationContext and GitTransaction already exist; consolidate effects only where needed and preserve their distinct allocation/existing-ticket responsibilities.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.


## Reactivation — 2026-11-03

The owner requested review of all parked work and execution of actionable tickets. This supersedes earlier blanket deferral. See [the parked-work review](../docs/parked-review-2026-11-03.md) for current scope and sequencing. Independent critic review is required before closure.

## Resolution (2026-11-04)

Explicit parent closure after all original criteria and critic acceptance, following children259,260,261 and earlier slices. Fresh gate523Rust/42Node pass,fmtclean/clippy0; one explicitly invoked helper ignored and one existing Windows Node skip. Installed source e755765 freshness verified. Durable evidence docs/evidence/git-effects-261.md. Owned intermediate scripts/logs recycled; final gate/install/deploy/status scratch retained through reconciliation then recycled. Shared target/dist retained for build/deploy reuse; unrelated artifacts preserved. Telemetry incident prior user state remains unknown. No release or hosted CI.

### Verification
1. ✓ Every mutating command uses one plan-and-apply effect path — "All capability-declared mutators use prepared FilePlan/effects::Phase routes; distinct Git transaction owners retained, no opaque callbacks. Source e755765; critic independently audited all original body criteria."
2. ✓ Dry run performs no persistent filesystem or Git mutation — "Fresh complete preview matrix snapshots files, directories, links, Git metadata and private state; private telemetry controls and partial-clone no-fetch control pass. JSON stderr intent records documented; stdout compatibility preserved."
3. ✓ The shared executor preserves atomic writes and push gating — "Source and critic audit confirm atomic_write/no-clobber publication and existing push.enabled-gated scoped Git/CAS/recovery owners. Fresh full mise gate523Rust/42Node passed with documented exclusions."
4. ✓ Table-driven tests cover no-write dry-run behavior — "Capability-linked table has29local routes with exact operation sets plus remote collision/stale refs, import/pending recovery, trusted process marker and private telemetry unit controls; docs/mutation-preview-coverage.md."
