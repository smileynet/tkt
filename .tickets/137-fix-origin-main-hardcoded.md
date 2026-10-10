---
id: "137"
title: "Fix origin/main hardcoded in race detection"
status: open
blocked_by: []
priority: high
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
no-ops. That breaks the headline race-safety guarantee for those setups. Resolve the upstream
once (the current branch's tracking ref first, then the selected remote's `HEAD` fallback),
and carry that result through each conflict, fetch, status, rebase, and publish decision that
needs it.

## Context

- **Location (#128 F7):** `src/mutation.rs:115`, `src/git.rs:183`; contrast `git.rs:34` (`has_remote`).
- **Contract:** README "race-safe — concurrent sessions get unique IDs automatically."
- **Boundary:** keep `GitTransaction` and `MutationContext` distinct. Put a small resolver in
  the git boundary and pass its typed result through existing workflows rather than merging
  unrelated abstractions.
- **Failure policy:** no-upstream and detached/local-only repositories must remain explicit,
  safe local workflows; a resolver failure must not silently turn an intended remote race check
  into a successful no-op.

## Acceptance criteria

- [ ] A single upstream resolver prefers the current branch's tracking ref and otherwise uses the selected remote's HEAD
- [ ] Conflict, fetch, status, rebase, and publish paths use that result rather than literal `origin/main`
- [ ] Repositories with `master`, a non-`origin` remote, no upstream, and detached HEAD have explicit, tested behavior
- [ ] Resolver failure cannot silently bypass a remote race check
- [ ] Regression/unit coverage exercises tracking, remote-HEAD fallback, non-origin remotes, no-upstream, and detached cases
- [ ] cargo fmt && cargo clippy --all-targets && cargo test pass clean
