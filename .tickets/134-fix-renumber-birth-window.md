---
id: "134"
title: "Fix renumber: enforce birth-window citation scan before apply"
status: done
blocked_by: ["204"]
priority: high
validation_criteria:
  - "renumber refuses when old ID is cited in other tickets"
  - "partial failure does not leave corpus in inconsistent state"
---

# Fix renumber: enforce birth-window citation scan before apply

> Source: #128 **F4** (P0, 2026-08-23 architecture audit). #128 is done; evidence + fix sketch below.

## What to build

`tkt renumber` must honor the documented birth-window contract: an id may only be renumbered
while it is uncited. Today renumber performs no citation scan, so a block-style citer keeps
pointing at the dead id after the rename, and a single unparseable `.md` aborts the operation
*after* renames are already applied — leaving the corpus in an inconsistent state. Renumber
must fully preflight the operation from a stable corpus snapshot: identify every potentially
affected file, parse and scan it, produce every rewritten byte sequence, then apply that fixed
plan. It must refuse before the first write if a citation exists or any potentially affected
ticket cannot be safely parsed; it must never discover work or skip a malformed file after
mutation begins.

## Context

- **Location (#128 F4):** `src/commands/renumber.rs:10-41`, `src/renumber.rs:92-150, 218-249` (confirm drift).
- **Contract:** `GLOSSARY.md` birth window — "cited ids are contracts."
- **Architecture decision (2026-10-10):** use a concrete, materialized `RenumberPlan`, not a
  best-effort streaming rewrite. The plan owns target paths, rewritten content, citation
  violations, and the affected-file set before apply starts. A failed preflight leaves the
  corpus byte-identical.

## Acceptance criteria

- [x] `renumber` refuses before writing when the old id is cited in another ticket's blocked_by or body
- [x] Block-style `blocked_by` citers are detected by the scan (not just inline arrays)
- [x] Preflight materializes all affected paths and replacement bytes before the first rename or write
- [x] An unparseable potentially affected file or citation violation leaves the entire corpus byte-identical
- [x] Apply consumes only the approved plan; it performs no new corpus discovery after the first write
- [x] Regression coverage proves cited-id refusal, malformed-file preflight refusal, and successful complete application
- [x] cargo fmt && cargo clippy --all-targets && cargo test pass clean

## Resolution (2026-10-19)

Renumber now materializes and validates every local ticket before mutating. Manual renumber rejects citations in dependencies or body text, while collision recovery reuses the preflighted rewrite plan without birth-window enforcement.

### Verification
1. ✓ renumber refuses when old ID is cited in other tickets — "cargo test test_renumber_refuses_one_item_block_style_citation_before_writing: passed; cited ID left all files unchanged"
2. ✓ partial failure does not leave corpus in inconsistent state — "cargo test test_renumber_refuses_malformed_ticket_before_writing: passed; malformed local ticket aborted before rename"
