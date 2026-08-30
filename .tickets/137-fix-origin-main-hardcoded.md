---
id: "137"
title: "Fix origin/main hardcoded in race detection"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "race detection works with master branch and non-origin remotes"
---

# Fix origin/main hardcoded in race detection

> Source: #128 **F7** (P1, 2026-08-23 architecture audit). #128 is done; evidence + fix sketch below.

## What to build

Race detection must work against the repository's actual upstream branch, not a hardcoded
`origin/main`. Today `mutation.rs:115` and `git.rs:183` assume `origin/main`, while
`git.rs:34` (`has_remote`) accepts any remote — so repos whose default branch is `master`,
or whose remote isn't named `origin`, still fetch and push but the conflict check silently
no-ops. That breaks the headline race-safety guarantee for those setups. Resolve the real
upstream ref (tracking branch or `origin/HEAD`) and use it for the conflict comparison.

## Context

- **Location (#128 F7):** `src/mutation.rs:115`, `src/git.rs:183`; contrast `git.rs:34` (`has_remote`).
- **Contract:** README "race-safe — concurrent sessions get unique IDs automatically."
- **Fix sketch (#128):** resolve the actual upstream ref (`git rev-parse --abbrev-ref origin/HEAD` or the branch's tracking ref) instead of literal `origin/main`.

## Acceptance criteria

- [ ] Race/conflict detection uses the resolved upstream branch, not literal `origin/main`
- [ ] A repo with a `master` default branch gets a real conflict check (not a silent no-op)
- [ ] A repo with a non-`origin` remote name is handled correctly
- [ ] Falls back gracefully when no upstream is configured (local-only repos unaffected)
- [ ] Regression/unit coverage for the upstream-ref resolution
- [ ] cargo fmt && cargo clippy --all-targets && cargo test pass clean
