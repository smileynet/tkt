---
id: "131"
title: "Fix block-style blocked_by parsed as empty + lint destroys deps"
status: done
blocked_by: []
priority: urgent
validation_criteria:
  - "blocked_by with YAML block list parsed correctly"
  - "tkt lint does not destroy unparseable blocked_by"
  - "bare scalar blocked_by (e.g. '01, 04') parsed correctly"
---

# Fix block-style blocked_by parsed as empty + lint destroys deps

## What to build

Fix two related data-loss paths in blocked_by handling:

**Bug A (parse layer):** `parse_blocked_by()` only handles inline `[...]` arrays. Block-style YAML lists (`- "01"`) are silently discarded — the dependency graph ignores these tickets' deps.

**Bug B (lint normalization):** `normalize_blocked_by()` rewrites single-line non-bracket values (e.g., `blocked_by: 01, 04`) as `[]`, destroying data. Block-style values survive lint (the `\n` check preserves them), but are still invisible to the dep graph due to Bug A.

## Context

- **Relevant files:** `src/core/ticket.rs` (parse_blocked_by, parse_string_array), `src/commands/lint.rs` (normalize_blocked_by, normalize_value)
- **Root cause:** `parse_blocked_by` requires `[...]` brackets; `parse_string_array` (used by tags/validation_criteria) already handles both formats

## Acceptance criteria

- [x] `blocked_by:\n  - "01"\n  - "03"` parses as vec!["01", "03"]
- [x] `blocked_by: 01, 04` (bare scalar) parses as vec!["01", "04"]
- [x] `tkt lint` on block-style deps normalizes to inline `["01", "03"]`
- [x] `tkt lint` on bare scalar deps normalizes to inline `["01", "04"]`
- [x] Existing inline-format blocked_by parsing unchanged
- [x] All existing tests pass

## Out of scope

- Tags/validation_criteria/requires parsing (already correct)
- CST-level roundtrip preservation
- New dependencies

## Resolution (2026-08-26)

parse_blocked_by handles inline, block-style, and bare scalar formats. normalize_blocked_by in lint parses bare scalars instead of discarding.

### Verification
1. ✓ blocked_by with YAML block list parsed correctly — "test parse_blocked_by_block_style passes"
2. ✓ tkt lint does not destroy unparseable blocked_by — "test: lint normalizes bare scalar via normalize_blocked_by refactor"
3. ✓ bare scalar blocked_by (e.g. '01, 04') parsed correctly — "test parse_blocked_by_bare_scalar passes"
