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
tags: ["architecture", "integrity"]
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
