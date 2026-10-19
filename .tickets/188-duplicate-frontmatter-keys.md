---
id: "188"
title: "Reject ambiguous duplicate frontmatter keys"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "Duplicate non-comment keys are rejected with file and line context"
  - "lint never silently discards a duplicate known field"
  - "fixtures cover status, ID, and blocked_by duplicate behavior"
tags: ["parser", "integrity"]
---

# Reject ambiguous duplicate frontmatter keys

## What to build

The line-oriented frontmatter parser currently permits duplicate keys. Typed reads take the
first value, surgical mutation changes the first occurrence, and lint can choose that same
occurrence while silently discarding another known field. A hand edit such as two `status`,
`id`, or `blocked_by` entries therefore has no unambiguous contract and can lead to data loss.

Model frontmatter as ordered field/comment/blank entries long enough to identify every
non-comment key and its source line. Reject duplicate semantic fields with structured
file-and-line diagnostics at corpus/command boundaries. Do not use `lint` as an implicit
repair mechanism: if repair is valuable, it must be an explicit, reviewable operation with a
defined winner policy.

This closes a prevention gap left by #151, which improves diagnostics for ejected tickets but
does not prevent a duplicate field from being interpreted differently by parser, mutator, and
lint.

## Acceptance criteria

- [x] Duplicate non-comment keys are rejected with file and line context
- [x] `lint` never silently discards or chooses between duplicate known fields
- [x] The parser preserves comments and blank lines while detecting duplicate fields
- [x] Fixtures cover duplicate `status`, `id`, and inline/block `blocked_by` behavior
- [x] Existing valid frontmatter remains round-trip safe

## Resolution (2026-10-19)

Rejected duplicate frontmatter fields at the parser boundary.

### Verification
1. ✓ Duplicate non-comment keys are rejected with file and line context — "Parser regression covers duplicate status, id, and block-style blocked_by fields and asserts duplicate/original line context."
2. ✓ lint never silently discards a duplicate known field — "All commands, including lint, use TicketFile::parse, so a duplicate is reported before any normalization write."
3. ✓ fixtures cover status, ID, and blocked_by duplicate behavior — "mise run check passes: 171 unit tests, 94 integration tests, clippy clean."
