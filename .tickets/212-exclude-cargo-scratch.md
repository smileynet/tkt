---
id: "212"
title: "Exclude tracked scratch artifacts from the published crate"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "Cargo package excludes all scratch files, including a seeded tracked .scratch/HANDOFF.md, while preserving required source and notices."
  - "A regression fails on the old Cargo exclusion list and passes on the repaired policy without registry publication."
  - "A fresh local cargo package builds successfully; archive inspection verifies identity and exclusions, repeated packaging preserves the hash, and the project gate passes."
tags: ["stream:tooling"]
---

# Exclude tracked scratch artifacts from the published crate

## What to build

TBD

## Acceptance criteria

- [ ] TBD
