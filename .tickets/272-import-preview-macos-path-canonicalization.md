---
id: "272"
title: "Canonicalize macOS temp paths in import preview test"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "cargo test commands::import::phase_tests::import_commit_and_recovery_previews_include_exact_owned_pair_paths -- --exact"
  - "mise run check"
tags: ["tooling"]
---
# Intent source

Discovered while executing #268 on 2026-11-08. The Clippy fix passed, but the required full gate exposed this independent macOS path-canonicalization failure.

# Observed problem

The focused command `cargo test commands::import::phase_tests::import_commit_and_recovery_previews_include_exact_owned_pair_paths -- --exact` fails in seconds. The expected and actual owned-path sets differ only because the temporary directory is represented as `/var/...` in some entries and its canonical `/private/var/...` form in others. This blocks the full Rust suite and therefore #268.

# What to build

Make the import commit/recovery preview test compare one consistent path representation on macOS without weakening its exact owned-path assertions. Preserve production behavior unless a product-side inconsistency is demonstrated by the test. Keep the fix scoped to canonicalization and add no platform-specific skip.

# Out of scope

The original newline-byte Clippy change, unrelated import behavior, and suppressing the test.

## Acceptance criteria

- [ ] The focused import preview test passes on macOS with exact owned-path coverage intact.
- [ ] The complete `mise run check` gate passes after both this fix and #268 are present.
- [ ] No platform skip or relaxed unordered subset assertion replaces the exact path comparison.
