---
id: "164"
title: "validate: recognize slug ids as canonical (stop false id-filename-mismatch)"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "when id equals the filename stem (slug id), validate does NOT emit id-filename-mismatch (test: validate::slug_id_canonical)"
  - "numeric-id corpuses still flag genuine mismatches"
  - "ADR records the decision on whether non-numeric ids are officially supported"
tags: ["compliance"]
---

# validate: recognize slug ids as canonical (stop false id-filename-mismatch)

## What to build

TBD

## Acceptance criteria

- [ ] TBD
