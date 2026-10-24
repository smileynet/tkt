---
id: "213"
title: "Use packaged VCS metadata instead of an unrelated ancestor Git hash"
status: done
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

- [x] Packaged builds identify the sealed source commit; unknown builds do not borrow ancestor hashes.
- [x] Ordinary/worktree identity and rebuild metadata are correct despite ambient overrides.
- [x] Seeded regressions, full Rust gate and a freshly packaged binary verify the fix.

Evidence: `node --test tools/release-tools.test.mjs` passes seven fixtures. The new real-Cargo fixture first failed on the old build script with an ancestor hash, then passed packaged/dirty/malformed/unknown metadata, ambient Git overrides, consecutive ordinary commits, packed-to-loose refs, linked commits and detached switches. `mise run check` passes fmt, clippy and all 296 Rust tests. Clean `cargo package --locked` at `b849d433450d63b4d8bf3757adeb4345018024f2` rebuilds inside grey-matter and reports `tkt 0.4.0 (b849d43)`, matching archive VCS metadata. Archive SHA-256: `018e36f0f937065259de8d3f1358825260d0dc6474d0b02b2c6508b7fe47c742`; 207970 bytes, no scratch. `cargo install --path . --locked` then `tkt --version` matches the source commit; `tkt validate --brief` reports zero findings without the obsolete config warning. This is source remediation evidence, not final approved-version candidate proof.

Build policy: packaged metadata is descriptive, not a provenance guarantee; malformed-present metadata yields unknown identity, and dirty metadata displays a suffix. Checkout discovery requires a local `.git` marker and canonical package/repository-root equality. Git overrides are removed; HEAD, refs, packed-refs and existing reftable storage paths are resolved by Git. Reftable storage is watched but the real transition fixtures cover the files backend. Sources: [Git worktree internals](https://git-scm.com/docs/git-worktree#_details), [Git repository environment](https://git-scm.com/docs/githooks#_description).

## Resolution (2026-10-24)

done: prefer validated Cargo VCS metadata and constrain sanitized live Git identity to the package checkout; track real worktree metadata for rebuilds

### Verification
1. ✓ A packaged build uses the .cargo_vcs_info.json commit, even when extracted inside another Git checkout; a non-Git build without metadata reports no fabricated hash. — "Real Cargo metadata fixture rejects ancestor fallback and malformed values; fresh packaged binary at b849d433450d63b4d8bf3757adeb4345018024f2 reports matching b849d43 inside grey-matter."
2. ✓ Ordinary and linked-worktree builds use their own commit and track the correct Git metadata for rebuilds; ambient Git repository overrides cannot redirect identity. — "Consecutive real builds cover ordinary, packed-to-loose, linked and detached refs with ambient GIT_DIR/WORK_TREE/config injection; all use their own commit."
3. ✓ Seeded regressions reproduce the old ancestor-hash bug and cover metadata validation; cargo fmt, clippy and all tests pass, and a rebuilt crate binary reports its sealed candidate commit. — "Old build script failed the ancestor-hash regression; seven Node fixtures, fmt, clippy and 296 Rust tests pass. Fresh package builds, archive metadata matches, installed CLI rebuilt and validates with zero findings."
