---
id: "39"
title: "Fix Windows path unit test (F1)"
status: done
blocked_by: []
priority: high
---

# Fix Windows path unit test (F1)

## Origin

Review ticket #38, finding F1.

## Problem

`telemetry::tests::project_slug_from_path` asserts `project_slug(Path::new("D:\\code\\game-research")) == "game-research"`. On Linux, `Path::file_name()` doesn't recognize `\` as a separator, so the whole string is returned as the filename. This makes `cargo test` red on the primary dev platform.

Pre-existing (present at 208b38b), but it blocks the verification gate.

## What to build

Either:
- `#[cfg(windows)]` around the Windows assertion, OR
- Make `project_slug` split on both `/` and `\` so it works cross-platform (preferred — the function itself should handle Windows paths on any host)

## Acceptance criteria

- [x] `cargo test --bin tkt telemetry::tests::project_slug_from_path` passes on Linux
- [x] Windows path input still produces the correct slug
- [x] Full `cargo test` passes (0 failures)

## Resolution (2026-08-05)

Made `project_slug` split on both `/` and `\` using `rsplit(|c| c == '/' || c == '\\')` instead of `Path::file_name()`. The existing test assertion now passes on Linux. All 40 unit tests green.
