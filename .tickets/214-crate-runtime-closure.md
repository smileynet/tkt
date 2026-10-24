---
id: "214"
title: "Limit the published crate to CLI runtime source and required notices"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "Real Cargo packaging includes required CLI source, build script, lockfile, license and user docs while excluding plugin and maintainer resources."
  - "A seeded Cargo regression fails with the previous broad package policy and passes with the explicit include policy; the full maintainer gate passes."
  - "A fresh clean crate builds and its contents, VCS identity and binary version are checked without registry publication."
tags: ["stream:tooling"]
---

# Limit the published crate to CLI runtime source and required notices

## What to build

The real crate inspected during release preparation contains `tools/`, plugin manifests, skills and Node maintainer metadata. No Rust source or build script uses include_str/include_bytes or loads those directories; they are independent plugin/maintainer distribution inputs. Ticket 212 excluded scratch but did not close the broader CLI package input set. Replace Cargo's broad exclusions with explicit CLI source/build/lock/license/README/changelog inputs, preserve generated Cargo metadata, and prove the actual package still builds. Extend the real Cargo fixture to seed tracked plugin/maintainer files and fail under the previous policy. This supports grey-matter preparation 67 and final audit 70; no registry publication is authorized.

Source evidence: `cargo package --locked` archive at grey-matter `.scratch/tkt-package-213/package/tkt-0.4.0.crate` (source `b849d43`); `rg` over src and build.rs finds no external embedded resource references. Use actual Cargo package listing/build behavior, not a regex-only check.

## Acceptance criteria

- [x] Required CLI source, build script, lockfile, notices and user docs are retained; plugin/maintainer files are excluded.
- [x] Real Cargo regression fails under the old policy and passes under the new policy; the full project gate passes.
- [x] Fresh clean package builds; archive closure, VCS metadata and binary version match the source commit; no upload occurs.

## Resolution (2026-10-24)

done: anchor the Cargo allowlist at package root; retain required CLI inputs and prove a clean extracted crate builds with matching source identity.

### Verification
1. ✓ Real Cargo packaging includes required CLI source, build script, lockfile, license and user docs while excluding plugin and maintainer resources. — "Clean cargo package at 62250e45bde3f1f012e0cacf907d2c76f5ecec21 contains 50 files: src, build script, Cargo metadata/lock, license and user docs only; no plugin or maintainer resources."
2. ✓ A seeded Cargo regression fails with the previous broad package policy and passes with the explicit include policy; the full maintainer gate passes. — "Real Cargo fixture reproduces broad-policy leakage and nested README/LICENSE/build.rs leakage; anchored policy passes. mise run check passes 7 Node checks, 296 Rust tests, fmt and clippy."
3. ✓ A fresh clean crate builds and its contents, VCS identity and binary version are checked without registry publication. — "cargo package --locked builds fresh extracted crate; archive VCS SHA matches 62250e4 and binary reports tkt 0.4.0 (62250e4). Archive SHA256 ef2a46f0754cb8521b967496538cd015eb2ba36871c6f83172cf53a12dd901c4. Durable evidence in grey-matter docs/agents/research/evidence/crate-runtime-closure-20261023.json; no upload."
