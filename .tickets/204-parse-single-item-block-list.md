---
id: "204"
title: "Parse single-item block-style blocked_by lists correctly"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "a one-item block-style blocked_by list parses as its id"
  - "tkt renumber refuses a cited id in a one-item block-style list"
tags: ["parser", "release"]
---

# Parse single-item block-style blocked_by lists correctly

## What to build

Treat a one-item block-style YAML list under `blocked_by` the same way as a multi-item
block list. Today `TicketFile::get` trims the leading newline; the parser then mistakes
the remaining `- "01"` text for a bare scalar and returns the literal `- "01` instead
of the referenced ID. That makes a real dependency invisible to frontier and
birth-window safety checks.

Source: discovered while implementing #134's required block-style citation regression
test on 2026-10-19. This is a parser-contract defect and directly blocks #134.

## Context

- Read `src/core/ticket.rs`, especially `TicketFile::get` and `parse_blocked_by`.
- Preserve support for inline arrays, bare references, and multi-item block lists.
- Do not weaken #134's citation scanner to accommodate this parser error.

## Acceptance criteria

- [x] A ticket containing `blocked_by:` followed by one `- "NN"` item exposes `NN` as
      its only typed dependency
- [x] The existing multi-item block-list behavior and bare/inline formats remain
      unchanged
- [x] #134's birth-window preflight rejects a one-item block-style citation

## Resolution (2026-10-19)

Single-item block-style dependencies now use the same list parser as multi-item lists, so typed dependencies, frontier computation, and renumber citation checks retain the referenced ID.

### Verification
1. ✓ a one-item block-style blocked_by list parses as its id — "cargo test block_style: passed the new single-item and existing multi-item parser regressions"
2. ✓ tkt renumber refuses a cited id in a one-item block-style list — "cargo test test_renumber_refuses_one_item_block_style_citation_before_writing: passed; public command rejected the citation before writes"
