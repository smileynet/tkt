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

Real `cargo package --locked` on approved initial candidate `8c4038574c2a5bb58086909859cd14b6071115f6` (0.5.0) created an archive containing `.scratch/HANDOFF.md`. The file is tracked, so Git ignore alone does not exclude it. Add `.scratch/` to Cargo package exclusions; retain the local handoff but remove the tracked scratch exception after routing any still-current decisions to the plan/tickets. Seed a tracked scratch fixture and test real Cargo listing/packaging semantics. Preserve source, notices, clean Git identity and no-upload operation. This blocks coordinated grey-matter preparation 67 and final audit 70 until closed.

Primary sources: [Cargo package metadata](https://doc.rust-lang.org/cargo/commands/cargo-package.html#cargo_vcs_infojson-format), [Cargo build-script rerun rules](https://doc.rust-lang.org/cargo/reference/build-scripts.html#change-detection). Local evidence is committed in grey-matter `docs/agents/research/evidence/registry-initial-package-findings-20261023.json`. No registry publication was run.

## Acceptance criteria

- [ ] Cargo excludes tracked scratch while retaining source/notices.
- [ ] Old exclusion policy fails a real Cargo fixture; repaired policy passes.
- [ ] Fresh package builds, archive exclusions/identity are inspected, replay hash matches and the project gate passes.
