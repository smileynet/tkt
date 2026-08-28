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

Discovered during the 2026-08-28 cross-project ticket audit. Five projects
(~170 tickets total: catalyst-mono, lacrosse-bosse-platform, codex_runner,
codex_runner-lbp, codex_runner-catalyst-mono) use **intentional slug ids**:
`id: cr-axcq` in file `cr-axcq.md`, `id: s2-overm-t001` in `s2-overm-t001.md`.
The id matches the filename stem — these are canonical, not defects.

But `tkt validate` emits `id-filename-mismatch` for every one of them because it
expects `{numeric-id}-{slug}.md`. Result: these projects show `fail` with 20-55
findings each, and the noise **drowns real findings** — you can't tell at a glance
whether a slug-id project has an actual problem.

These ids are cited contracts (renumbering breaks references — birth-window rule),
so migrating them is not the answer. tkt should recognize the slug-id scheme.

Fix:
- When `id:` equals the filename stem (whole filename minus `.md`), treat it as
  canonical — do NOT emit id-filename-mismatch.
- Keep flagging genuine mismatches in numeric-id corpuses (id `"04"` in file `07-foo.md`).
- Record the decision in an ADR: does tkt officially support non-numeric / slug ids
  as a first-class scheme? (Affects `tkt new` slug handling, renumber, and docs.)

## Acceptance criteria

- [ ] when `id` equals the filename stem, validate does NOT emit id-filename-mismatch
- [ ] numeric-id corpuses still flag genuine id/filename mismatches
- [ ] slug-id projects (codex_runner, catalyst-mono) validate without the mismatch noise
- [ ] ADR records whether non-numeric ids are officially supported
- [ ] regression test (validate::slug_id_canonical)
