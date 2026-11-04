---
id: "263"
title: "Scope Cargo identity and cargo-dist checks to affected work"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "Independently confirm current aggregate selection and native prerequisites before changing the gate."
  - "Unrelated tooling work retains cheap static, ownership, credential, workflow security and seal integrity blockers without repeated identity rebuilds or installed cargo-dist."
  - "Real identity and generator regression tests remain runnable for affected source and release verification; optional full pre-push remains opt-in."
tags: ["tooling"]
---
Scoped gate review throughf0cf085 following ticket262; related historical tickets216 and219 preserve the reasons these regressions matter. Reviewer: Codex and independent review. Run: gate-review-20261104-gm11cbd1e-tktf0cf085. Confirmation status: unconfirmed independently. Existing scope/cost findings, not regressions from262: mise.toml15 runs the Node aggregate on every full check. tools/release-tools.test.mjs110 creates a fresh Cargo target and repeatedly rebuilds source identity (14.2s in this review); scope to build.rs/identity inputs and release verification while retaining cheap ownership/credential/publication safety. tools/release-ci.test.mjs30 regenerates real dist workflows twice; tools/generate-release-workflow.mjs70 requires cargo-dist0.32.0. That installed-tool prerequisite can block unrelated work even though it took1.3s here. Separate real generation from cheap workflow topology/security and seal tests. Keep real regression fixtures rather than delete them. Native-Claude split itself is intact. Fresh aggregate43/43 passed without skips in17.1s. tools/pre-push-check.mjs intentionally invokes full mise check only after explicit opt-in and is not installed here; preserve opt-in. No implementation changes or full Rust run in this review.

## Resolution (2026-11-04)

done: cheap safety always runs; real identity and cargo-dist suites selected by affected inputs, explicit base, conservative unknown scope or forced release mode. Gate/task edits remain cheap. Retired permanent frozen-migration exact-text obligation for later guidance edits while preserving historical audit. Verified, committed, pushed and redeployed both packs. Task helper scripts sent to Trash; existing dist496KiB preserved.

### Verification
1. ✓ Independently confirm current aggregate selection and native prerequisites before changing the gate. — "Confirmed source selection findings before edits: routine aggregate repeated Cargo identity builds and installed cargo-dist generation. Preserved real regression cases in separate suites; reviewed historical216/219 context."
2. ✓ Unrelated tooling work retains cheap static, ownership, credential, workflow security and seal integrity blockers without repeated identity rebuilds or installed cargo-dist. — "Final cheap tooling aggregate46/46 passed, no skips,2.95s on synced checkout; default selected identity:false,generation:false. Additional cheap run with dist/cargo-run invocations rejected passed46/46. Selector fixtures cover gate edits, affected paths, dirty/deleted/untracked input, invalid/unknown base and foreign Git environment overrides. Six changed guidance docs66 local links pass; diff/dashboard checks pass."
3. ✓ Real identity and generator regression tests remain runnable for affected source and release verification; optional full pre-push remains opt-in. — "node tools/check-tools.mjs --release:48/48 pass,no skips,12.66s. Mise release dry-run preserves native selection plus fmt/clippy/Rust pipeline; shared release product fixture9/9 and22-module inventory pass. Optional full pre-push remains opt-in. Commit13d2a98 pushed; portfolio deployment refreshed both packs and strict validation passed. Frozen historical guidance-text audit is now explicitly on-demand; no currentText override or new mandatory gate. No Rust product code changed or full Rust/platform pass claimed."
