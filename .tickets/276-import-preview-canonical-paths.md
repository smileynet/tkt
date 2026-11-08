---
id: "276"
title: "Normalize macOS temporary paths in import preview test"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "Import preview path assertions and no-write checks pass on macOS and in the complete Rust suite"
tags: ["tooling", "kind:bug", "stream:quality"]
---
# Normalize macOS temporary paths in import preview test

## Intent source

Sam approved the 2026-11-08 repository-audit fix plan. The focused unit test fails independently on the current macOS host. This is a new path-normalization gap adjacent to, but distinct from, the fixture repairs in #256.

## Observed problem

`import_commit_and_recovery_previews_include_exact_owned_pair_paths` builds expected paths from `tempfile::TempDir::path()` using `/var/...`, while `GitTransaction` reports canonical `/private/var/...` paths. The owned path sets are semantically equal but fail string/path comparison.

## What to build

Normalize the test fixture root before constructing expected paths, or compare an equivalently normalized path set. Preserve strict coverage of the exact owned files and the existing no-write assertions; do not hide missing or additional effects.

## Acceptance criteria

- [ ] The focused import preview unit test passes on macOS without skipping assertions.
- [ ] Commit, rollback, finish-rollback, and rebase previews retain their exact owned-path assertions.
- [ ] Preview still creates no ticket, snapshot, journal, or allocation-lock changes.
- [ ] The complete Rust test suite passes after independent blockers are repaired.

## Out of scope

Changing production import behavior when only path spelling differs, or broad fixture refactoring.
