---
id: "272"
title: "Canonicalize macOS temp paths in import preview test"
status: done
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

- [x] The focused import preview test passes on macOS with exact owned-path coverage intact.
- [x] The complete `mise run check` gate passes after both this fix and #268 are present.
- [x] No platform skip or relaxed unordered subset assertion replaces the exact path comparison.

## Resolution (2026-11-08)

Canonicalized the import preview temporary root on macOS while preserving exact path assertions. Committed and pushed ecb85fd; installed tkt 0.4.0 (ecb85fd) and verified binary freshness. No scratch artifacts retained.

### Verification
1. ✓ cargo test commands::import::phase_tests::import_commit_and_recovery_previews_include_exact_owned_pair_paths -- --exact — "Focused import preview test passed: 1 test, exact owned-path set preserved."
2. ✓ mise run check — "mise run check passed at ecb85fd: 48 tooling tests, formatting, warning-free Clippy, 216 unit tests, and all integration suites."
