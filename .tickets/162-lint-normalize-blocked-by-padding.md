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

Discovered during a cross-project ticket audit (~/code, 20 tkt repos, 2026-08-28).
`dangling-blocked-by` from **id-format mismatch** was one of the two dominant
compliance errors across the corpus, and it recurs — the same repo re-introduced
it in newly-created tickets after an earlier hand-fix.

Two shapes observed:
1. **Padding mismatch** — `blocked_by: ["5"]` when ticket ids are zero-padded
   (`id: "05"`). Falsely blocks the dependent (ref resolves to nothing).
   Seen in gdhelper-harness (8 tickets, then 4 more from upstream) and codex_runner variants.
2. **Slug ref** — `blocked_by: ["004-spike-pi-lmstudio-integration"]` instead of
   the bare id `"004"`. Seen in local-models (8 tickets, masked until id: was added).

Today `tkt lint --check` reports "all files canonical" while these dangling-by-format
errors exist — **lint and validate disagree**. Lint should own this normalization.

Fix:
- `tkt lint` normalizes each `blocked_by` value: zero-pad numeric ids to the corpus's
  id width; strip `-slug` suffix from `NNN-slug` refs to the bare `NNN`.
- Only rewrite when the normalized ref resolves to a real ticket (deterministic, unique).
- Leave genuinely-dangling refs (no matching ticket) alone — those are real errors validate keeps.
- `tkt validate --fix` gains the same resolution so a validate pass can self-heal.

## Acceptance criteria

- [ ] `tkt lint --fix` pads single-digit blocked_by ids to corpus id width
- [ ] `tkt lint --fix` strips `-slug` suffix from `NNN-slug` blocked_by refs to bare `NNN`
- [ ] normalization only applies when the result resolves to an existing ticket
- [ ] `tkt lint --check` reports non-canonical blocked_by (lint/validate no longer disagree)
- [ ] `tkt validate --fix` resolves the same dangling-by-format cases
- [ ] regression test covers padding + slug-strip (lint::normalize_blocked_by_padding)
