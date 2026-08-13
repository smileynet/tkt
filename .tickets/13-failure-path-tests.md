---
id: "13"
title: "failure-path and race integration tests"
status: done
blocked_by: ["07"]
---

# Failure-path and race integration tests

## What to build

Current tests cover only happy paths. The core differentiator (push-to-claim atomicity) has zero test coverage. Add integration tests for failure scenarios.

### Test cases needed

1. **Push rejection + successful retry** — two clones, unrelated upstream commit, verify rebase+push works
2. **Second push rejection** — force both retries to fail, verify nonzero exit and meaningful error
3. **Auth/remote failure** — configure unreachable remote, verify no rebase attempted, error propagated
4. **Competing allocation** — two clones allocate same ID, verify loser gets different ID or fails cleanly
5. **Stale claim** — ticket closed on remote between local read and push, verify claim detects conflict
6. **Rebase conflict** — same field edited by two clones, verify conflict reported (not silent success)
7. **No remote configured** — verify commands work locally with appropriate messaging
8. **Argument-boundary safety** — paths/titles with spaces, quotes, shell metacharacters pass through safely

### Approach

Use the existing tempdir+bare-repo pattern. Create helper functions for multi-clone setups. Use `pre-push` hooks or interleaved operations for deterministic race simulation.

## Acceptance criteria

- [x] At least 6 new integration tests covering failure paths
- [x] At least 1 test with two competing clones
- [x] Push failure test verifies no pull --rebase on non-race errors
- [x] All new tests pass on CI (no timing dependencies)
- [x] Tests verify exit codes, not just stdout content
