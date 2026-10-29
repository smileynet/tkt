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

Discovered during the 2026-08-28 cross-project ticket audit. In the `local-models`
repo, **all 21 tickets were hand-authored with no `id:` frontmatter field**, making
them unparseable: `tkt validate` → `fail (21)`, every finding
`[unparseable] missing required field: id`.

tkt derives the id from the filename at read time (in-memory) but does **not** persist
it, so raw files still fail for any external reader and for validate itself. The repair
today is manual (insert `id: "NNN"` from the `NNN-slug.md` filename prefix into each file).

This should be a one-command fix. `tkt lint --fix` should backfill `id:` from the
filename prefix when the frontmatter lacks it — deterministic and safe (the filename
prefix is already the canonical id source tkt uses in memory).

Related: this is the birth of hand-authored / migrated corpuses. `tkt doctor --fix`
could alternatively adopt orphaned tickets, but lint is the natural home since it
already normalizes frontmatter style.

## Acceptance criteria

- [ ] `tkt lint --fix` inserts `id:` derived from the `NNN-slug` filename prefix when frontmatter lacks `id`
- [ ] the inserted id is a quoted string matching tkt's in-memory derivation
- [ ] hand-authored tickets with no `id:` become parseable + canonical after `lint --fix`
- [ ] `tkt validate` no longer reports unparseable missing-id for fixable files
- [ ] no change to files that already have an `id:`
- [ ] regression test (lint::backfill_missing_id)
