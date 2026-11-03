---
id: "191"
title: "Centralize command mutation effects and dry-run policy"
status: backlog
blocked_by: ["136"]
priority: medium
validation_criteria:
  - "Every mutating command uses one plan-and-apply effect path"
  - "Dry run performs no persistent filesystem or Git mutation"
  - "The shared executor preserves atomic writes and push gating"
  - "Table-driven tests cover no-write dry-run behavior"
tags: ["architecture", "integrity", "kind:enhancement", "triage:ready-for-agent"]
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

- [ ] Every mutating command enumerates filesystem, Git, and publish effects through one plan-and-apply path
- [ ] Dry run performs no persistent filesystem write, Git mutation, marker creation, commit, or push
- [ ] The executor preserves `atomic_write` where appropriate and respects `push.enabled`
- [ ] Existing command output and error contracts remain compatible
- [ ] Table-driven tests cover every mutator's no-write dry-run behavior

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `architecture`.

Deferred consolidation after completed #136. MutationContext and GitTransaction already exist; consolidate effects only where needed and preserve their distinct allocation/existing-ticket responsibilities.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.
