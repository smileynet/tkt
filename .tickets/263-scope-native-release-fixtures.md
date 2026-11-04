---
id: "263"
title: "Scope Cargo identity and cargo-dist checks to affected work"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "Independently confirm current aggregate selection and native prerequisites before changing the gate."
  - "Unrelated tooling work retains cheap static, ownership, credential, workflow security and seal integrity blockers without repeated identity rebuilds or installed cargo-dist."
  - "Real identity and generator regression tests remain runnable for affected source and release verification; optional full pre-push remains opt-in."
tags: ["tooling"]
---
Scoped gate review throughf0cf085 following ticket262; related historical tickets216 and219 preserve the reasons these regressions matter. Reviewer: Codex and independent review. Run: gate-review-20261104-gm11cbd1e-tktf0cf085. Confirmation status: unconfirmed independently. Existing scope/cost findings, not regressions from262: mise.toml15 runs the Node aggregate on every full check. tools/release-tools.test.mjs110 creates a fresh Cargo target and repeatedly rebuilds source identity (14.2s in this review); scope to build.rs/identity inputs and release verification while retaining cheap ownership/credential/publication safety. tools/release-ci.test.mjs30 regenerates real dist workflows twice; tools/generate-release-workflow.mjs70 requires cargo-dist0.32.0. That installed-tool prerequisite can block unrelated work even though it took1.3s here. Separate real generation from cheap workflow topology/security and seal tests. Keep real regression fixtures rather than delete them. Native-Claude split itself is intact. Fresh aggregate43/43 passed without skips in17.1s. tools/pre-push-check.mjs intentionally invokes full mise check only after explicit opt-in and is not installed here; preserve opt-in. No implementation changes or full Rust run in this review.