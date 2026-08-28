---
id: "163"
title: "lint --fix: backfill missing id from filename prefix"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "tkt lint --fix inserts id: derived from filename NNN-slug prefix when frontmatter lacks id (test: lint::backfill_missing_id)"
  - "hand-authored tickets with no id: field become parseable + canonical after lint --fix"
  - "tkt validate no longer reports unparseable missing-required-field-id for fixable files"
tags: ["compliance"]
---

# lint --fix: backfill missing id from filename prefix

## What to build

TBD

## Acceptance criteria

- [ ] TBD
