---
id: "219"
title: "Make release generation and deployment fixtures portable on Windows"
status: in_progress
blocked_by: []
priority: high
validation_criteria:
  - "Windows Node release and workflow-generation fixtures pass without skipping generator or package exclusion checks"
  - "Deployment integration tests invoke the canonical Node deployer on Windows and preserve duplicate/foreign ownership assertions"
  - "A full hosted Windows check and the existing macOS/Linux gates pass with exact source evidence"
tags: ["stream:tooling"]
---

# Make release generation and deployment fixtures portable on Windows

## What to build

Windows maintainers can run the same release safety checks and duplicate-loading
fixtures as macOS/Linux maintainers. Preserve real cargo-dist generation and crate
exclusion assertions rather than skipping the failing checks.

## Intent source and failure evidence

Found during grey-matter release compatibility ticket 69, while preparing the
user-approved tkt 0.5.0 candidate. This blocks the coordinated final audit/staging.
Frozen candidate: `b2425c93f94a0b56a87cb6496ecbadaf381922bf`.

[Windows run 40079658409](https://github.com/smileynet/tkt/actions/runs/40079658409)
checks out that exact candidate; the probe workflow lives separately at
`fb9b2f9a17e28a92e0eb08d49f59ff4b4106e7a7`, branch
`codex/probe-tkt-windows-b2425c9`. It preserves Git file bytes, installs Node 22,
stable Rust/fmt/clippy, mise and cargo-dist 0.32.0, installs locked npm dependencies
and fetches Cargo dependencies before running checks.

Observed results: Node 34 passed / 2 failed / 1 skipped; Cargo fmt and Clippy pass;
191 Rust unit tests pass, then all six deploy_skills integration tests fail.

- `node --test tools/release-ci.test.mjs`: real generation fails inside its
  temporary copy with cargo-dist reporting no workspace/Cargo.toml. Diagnose the
  copied manifest and subprocess environment; the root cause is not established.
- `node --test tools/release-tools.test.mjs`: Cargo package fixture fails
  `required src/main.rs`. Inspect the actual Windows list before changing assertions;
  path separators are a hypothesis, not yet verified.
- `cargo test --test deploy_skills`: `Command::new("bash")` resolves Windows WSL
  bash, which reports no installed Linux distribution. These six fixtures exercise
  canonical deployer behavior, so invoke Node directly for portable coverage.

Earlier [run 40079229283](https://github.com/smileynet/tkt/actions/runs/40079229283)
also lacked mise/dist and converted checkout files to CRLF. Those probe setup
failures are resolved in the follow-up: the ownership inventory and mise wrapper
checks now pass. LF-to-CRLF conversion of the Git blob exactly reproduced the
original hash mismatch; no candidate source mismatch was found.

## Context and boundaries

Read `tools/generate-release-workflow.mjs`, `tools/release-ci.test.mjs`,
`tools/release-tools.test.mjs`, `tests/deploy_skills.rs`, and `tools/deploy.mjs`.
The deployer is vendored from grey-matter; change its harness behavior at the
canonical source only. This ticket should need generator/fixture portability,
not changes to published CLI behavior. Do not publish, tag, or advance stable.

Group verified corrections before regenerating the affected candidate. Source
changes invalidate its old candidate identity; preserve old reports as historical
and repeat affected source/artifact/product/native gates for the replacement.

## Acceptance criteria

- [ ] Windows Node release and workflow-generation fixtures pass without skipping generator or package exclusion checks.
- [ ] Deployment integration tests invoke the canonical Node deployer on Windows and preserve duplicate/foreign ownership assertions.
- [ ] A full hosted Windows check and the existing macOS/Linux gates pass with exact source evidence.
