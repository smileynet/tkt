---
id: "178"
title: "Restore automated CI gate for pushes and pull requests"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "CI runs mise run check on push and pull request events"
  - "CI uses a supported Rust toolchain with rustfmt and clippy"
  - "Repository documentation distinguishes the automated gate from local verification"
tags: ["release", "quality"]
---

# Restore automated CI gate for pushes and pull requests

## Context

The local `mise run check` gate is healthy, but `.github/workflows/ci.yml` is
manual-only (`workflow_dispatch`). A push or pull request therefore has no automatic
format, lint, or test verification. The release workflow exists, but it is not a
substitute for a normal change gate.

## What to build

Enable the existing CI workflow for `push` and `pull_request` events while retaining
manual dispatch. Run the canonical `mise run check` command (or an exactly equivalent
toolchain-pinned invocation) with `DO_NOT_TRACK=1`. Document the relationship between
the automatic CI gate and the local pre-commit check.

## Acceptance criteria

- [ ] CI runs `mise run check` on push and pull request events
- [ ] CI installs a supported Rust toolchain including rustfmt and Clippy
- [ ] CI runs tests with `DO_NOT_TRACK=1`
- [ ] Contributor guidance distinguishes the automated CI gate from local verification
