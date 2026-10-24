---
id: "212"
title: "Exclude tracked scratch artifacts from the published crate"
status: done
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

- [x] Cargo excludes tracked scratch while retaining source/notices.
- [x] Old exclusion policy fails a real Cargo fixture; repaired policy passes.
- [x] Fresh package builds, archive exclusions/identity are inspected, replay hash matches and the project gate passes.

Evidence: `node --test tools/release-tools.test.mjs` passes six fixtures, including a committed scratch seed that fails with the old policy and passes with the actual Cargo exclusion list. `mise run check` passes Node, fmt, clippy and 296 Rust tests. Two clean `cargo package --locked` runs at `a924678c509da445050514b072fa58956f1e7398` build successfully and produce identical archive SHA-256 `cf98292d07b4c7f6eabadf0809b5f183432573c42a94f3fa2031c685b303f6e1` (206021 bytes, 100 files). Archive inspection excludes all scratch files, retains source/LICENSE, and matches clean Cargo VCS metadata to that commit. The local handoff remains on disk, untracked; its durable ADR/plan decisions already have committed owners. Binary version identity remains ticket 213. Nothing was uploaded.

## Resolution (2026-10-24)

done: exclude scratch in Cargo policy and untrack the obsolete handoff while retaining its local copy; binary identity remains tracked separately in 213

### Verification
1. ✓ Cargo package excludes all scratch files, including a seeded tracked .scratch/HANDOFF.md, while preserving required source and notices. — "Actual Cargo fixture excludes tracked scratch and retains src/main.rs and LICENSE; local handoff retained untracked."
2. ✓ A regression fails on the old Cargo exclusion list and passes on the repaired policy without registry publication. — "Real Cargo fixture reproduces the old policy leak and passes with the repaired policy; six Node fixtures pass."
3. ✓ A fresh local cargo package builds successfully; archive inspection verifies identity and exclusions, repeated packaging preserves the hash, and the project gate passes. — "mise run check passes fmt, clippy and 296 Rust tests; two clean packages at a924678 have matching VCS metadata and identical SHA-256 cf98292d07b4c7f6eabadf0809b5f183432573c42a94f3fa2031c685b303f6e1. No uploads."
