---
id: "220"
title: "Make post-commit push rejection fixture deterministic"
status: in_progress
blocked_by: []
priority: high
validation_criteria:
  - "The integration test rejects only the push after a successful fetch and local close commit, then asserts preserved state and recovery output"
  - "Full local and hosted Linux checks pass without recursive remote permission mutation"
tags: ["stream:tooling"]
---

# Make post-commit push rejection fixture deterministic

## What to build

The post-commit failure regression reaches the product assertion reliably: fetch
succeeds, close writes and commits locally, the remote rejects the push, and the
CLI reports recoverable failure while retaining the commit and ticket change.

## Intent source and evidence

Found while verifying release blocker 219 in
[Linux CI 40081307626](https://github.com/smileynet/tkt/actions/runs/40081307626)
on `c0359f8c9975a9b4c172f5a5f1c5f1f9e83de16d`. The full Node, format, Clippy,
unit and deploy tests pass; the integration group fails 1 of 96 at the recursive
remote permission setup's metadata lookup, before the post-commit assertion.
This blocks the coordinated grey-matter 69/70 release verification.

Local deterministic reproduction: create an owned Git template directory whose
`hooks/inactive.sample` is a dangling symlink. Run
`GIT_TEMPLATE_DIR=<that directory> cargo test --test integration test_push_failure_after_commit_preserves_local_state -- --exact`.
The existing test fails with the same ENOENT at metadata lookup. This demonstrates
the fixture's sensitivity to template contents; it does not establish which exact
entry disappeared or was dangling on the hosted runner.

## Context

Read `tests/integration.rs`, specifically `setup_repo`,
`test_push_failure_after_commit_preserves_local_state`, and its single-use
`walk_dir` helper. Reject the push using an executable, fixture-owned
`pre-receive` hook instead of changing permissions throughout a remote snapshot.
Keep the successful preflight fetch, preserved local commit/file assertions,
exit 2 and actionable recovery message. Do not weaken or skip the regression.
The runtime CLI and publication behavior need no change.

## Acceptance criteria

- [ ] The integration test rejects only the push after a successful fetch and local close commit, then asserts preserved state and recovery output.
- [ ] Full local and hosted Linux checks pass without recursive remote permission mutation.


## Implemented and locally verified

The remote now has a fixture-owned executable `pre-receive` hook that returns 1;
its hook path is configured explicitly on that bare remote. Recursive chmod and
the single-use directory walker are removed. All existing exit-code, actionable
recovery, retained-commit and retained-ticket assertions remain.

The dangling-template reproduction changed from exit 101 (ENOENT at fixture setup)
to exit 0 (one passed test). A fresh `mise run check` passes 37 Node tests, fmt,
Clippy and all 296 Rust tests. Hosted Linux verification remains required.
