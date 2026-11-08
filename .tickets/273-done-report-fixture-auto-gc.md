---
id: "273"
title: "Disable automatic Git GC in done-report fixtures"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "cargo test --test done_report oversized_tree_listing_never_produces_dates_from_a_partial_map -- --exact"
  - "cargo test --test done_report report_lookup_budget_retains_remaining_rows_with_unknown_dates -- --exact"
  - "mise run check"
tags: ["tooling"]
---
# Intent source

Discovered while executing #268 after #272 fixed the first full-suite blocker on 2026-11-08.

# Observed problem

Both `cargo test --test done_report oversized_tree_listing_never_produces_dates_from_a_partial_map -- --exact` and `cargo test --test done_report report_lookup_budget_retains_remaining_rows_with_unknown_dates -- --exact` fail their unchanged-files assertions. A byte-map comparison shows that loose Git objects become a pack and `.git/info/refs`, `.git/objects/info/packs`, and the multi-pack-index change. This is automatic Git maintenance triggered by the large fixtures, not report output mutation.

# What to build

Disable automatic Git garbage collection in the shared done-report fixture so these tests measure report behavior rather than Git maintenance timing. Preserve the exact before/after snapshot assertions and all history-budget expectations. Do not skip the tests or exclude product-created files from the snapshots.

# Out of scope

Changing report history limits, the Clippy comparison, import path canonicalization, or global Git configuration.

## Acceptance criteria

- [x] Both named done-report tests pass individually with their exact snapshot assertions unchanged.
- [x] The full `mise run check` gate passes with the other blocker fixes present.
- [x] Only fixture-local Git configuration changes; no user or global Git setting is modified.

## Resolution (2026-11-08)

Disabled fixture-local automatic GC and explicitly packed the two large fixture repositories before their baseline snapshots. Product behavior and exact snapshot assertions remain unchanged. Committed and pushed ecb85fd; deployed binary freshness verified. Removed temporary diagnostic scripts and copied logs.

### Verification
1. ✓ cargo test --test done_report oversized_tree_listing_never_produces_dates_from_a_partial_map -- --exact — "oversized_tree_listing_never_produces_dates_from_a_partial_map passed individually after explicit baseline packing."
2. ✓ cargo test --test done_report report_lookup_budget_retains_remaining_rows_with_unknown_dates -- --exact — "report_lookup_budget_retains_remaining_rows_with_unknown_dates passed individually with its exact snapshot assertion unchanged."
3. ✓ mise run check — "mise run check passed at ecb85fd, including all 11 done-report tests and the complete Rust gate."
