---
id: "162"
title: "lint/validate: normalize blocked_by id padding and slug refs"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "tkt lint --fix pads single-digit blocked_by ids to corpus width (test: lint::normalize_blocked_by_padding)"
  - "tkt validate --fix resolves dangling-blocked-by when unique padding/slug-strip makes ref valid"
  - "tkt lint --check reports non-canonical blocked_by (no longer disagrees with validate)"
tags: ["compliance"]
---

# lint/validate: normalize blocked_by id padding and slug refs

## What to build

TBD

## Acceptance criteria

- [ ] TBD
