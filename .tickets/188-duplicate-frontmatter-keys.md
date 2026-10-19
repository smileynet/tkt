---
id: "188"
title: "Reject ambiguous duplicate frontmatter keys"
status: in_progress
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

- [ ] Duplicate non-comment keys are rejected with file and line context
- [ ] `lint` never silently discards or chooses between duplicate known fields
- [ ] The parser preserves comments and blank lines while detecting duplicate fields
- [ ] Fixtures cover duplicate `status`, `id`, and inline/block `blocked_by` behavior
- [ ] Existing valid frontmatter remains round-trip safe
