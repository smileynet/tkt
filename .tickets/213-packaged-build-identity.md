---
id: "213"
title: "Use packaged VCS metadata instead of an unrelated ancestor Git hash"
status: in_progress
blocked_by: []
priority: high
validation_criteria:
  - "A packaged build uses the .cargo_vcs_info.json commit, even when extracted inside another Git checkout; a non-Git build without metadata reports no fabricated hash."
  - "Ordinary and linked-worktree builds use their own commit and track the correct Git metadata for rebuilds; ambient Git repository overrides cannot redirect identity."
  - "Seeded regressions reproduce the old ancestor-hash bug and cover metadata validation; cargo fmt, clippy and all tests pass, and a rebuilt crate binary reports its sealed candidate commit."
tags: ["stream:tooling"]
---

# Use packaged VCS metadata instead of an unrelated ancestor Git hash

## What to build

The unpacked Cargo build of initial candidate `8c4038574c2a5bb58086909859cd14b6071115f6` reports `tkt 0.5.0 (4e13504)`: build.rs runs unscoped git discovery inside an archive nested beneath grey-matter. Prefer Cargo `.cargo_vcs_info.json` for packaged builds; validate its fields without claiming it proves provenance. For Git checkouts, require the package to belong to the expected repository root, clear ambient repository overrides and watch actual Git metadata for ordinary/linked worktrees. Unknown non-Git builds must omit the hash rather than borrow an ancestor. Cover malformed metadata, nested repos, linked worktrees and rebuild/watch behavior. This blocks coordinated grey-matter preparation 67 and final audit 70 until closed.

Primary sources: [Cargo package metadata](https://doc.rust-lang.org/cargo/commands/cargo-package.html#cargo_vcs_infojson-format), [Cargo build-script rerun rules](https://doc.rust-lang.org/cargo/reference/build-scripts.html#change-detection). Local evidence is committed in grey-matter `docs/agents/research/evidence/registry-initial-package-findings-20261023.json`. No registry publication was run.

## Acceptance criteria

- [ ] Packaged builds identify the sealed source commit; unknown builds do not borrow ancestor hashes.
- [ ] Ordinary/worktree identity and rebuild metadata are correct despite ambient overrides.
- [ ] Seeded regressions, full Rust gate and a freshly packaged binary verify the fix.
