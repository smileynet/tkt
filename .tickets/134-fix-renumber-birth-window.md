---
id: "134"
title: "Fix renumber: enforce birth-window citation scan before apply"
status: open
blocked_by: []
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
should (a) pre-flight scan for citations (remote tree + prose bodies + blocked_by) and refuse
(or require an explicit override) when the old id is cited, and (b) make its apply phase
skip-with-warning on unparseable files, matching `load_corpus` crash-consistency behavior.

## Context

- **Location (#128 F4):** `src/commands/renumber.rs:10-41`, `src/renumber.rs:92-150, 218-249` (confirm drift).
- **Contract:** `.memory/CONTEXT.md` birth window — "cited ids are contracts."
- **Fix sketch (#128):** pre-flight citation scan (warn or require force); Phase 2 skips unparseable files with a warning instead of aborting mid-plan.

## Acceptance criteria

- [ ] `renumber` refuses (or requires override) when the old id is cited in another ticket's blocked_by or body
- [ ] Block-style `blocked_by` citers are detected by the scan (not just inline arrays)
- [ ] A single unparseable file no longer leaves the corpus half-renamed — apply is crash-consistent (skip-with-warning)
- [ ] Regression test covering a cited-id refusal and a mid-plan unparseable file
- [ ] cargo fmt && cargo clippy --all-targets && cargo test pass clean
