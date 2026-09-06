---
id: "179"
title: "tkt close: per-criterion --evidence requirement is undiscoverable; error omits count + syntax"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "tkt close error names the exact number of criteria needing evidence AND shows the positional / N=text syntax to satisfy them"
  - "docs/help make explicit that --evidence is required once per validation criterion (repeatable)"
tags: ["cli"]
---

# tkt close: per-criterion --evidence requirement is undiscoverable; error omits count + syntax

## Problem

`tkt close` requires **one `--evidence` per validation criterion**, but nothing surfaces that
contract until you fail, and the failure message doesn't tell you (a) how many criteria need
evidence or (b) the syntax to satisfy them. A user with a valid, ready-to-close ticket gets a
terse rejection and no path forward — which reads as "tkt close is broken" rather than "supply
N evidence values."

Related but distinct from **#178** (`close --evidence` crashes the text-mode writer): #178 is a
crash on the *write*; this is the *validation error's quality/discoverability* on the way in.
Both together explain a downstream misdiagnosis (see Impact).

## Repro (tkt 0.3.1, macOS, dry-run — no mutation)

A ticket with 3 `validation_criteria`, closed with a single `--evidence`:

```bash
tkt close 316 --check-all --resolution "x" --evidence "x" --dry-run
# → tkt: ✗ --evidence: criteria 2, 3 have no evidence (provide positional or named evidence for each)
```

Supplying one `--evidence` per criterion works:

```bash
tkt close 316 --check-all --resolution "x" --evidence e1 --evidence e2 --evidence e3 --dry-run
# → Would close 316 … Verification: 3 criteria with evidence
```

The requirement (one evidence per criterion) is correct policy; the **problem is it's
undiscoverable**: the error names *which* criteria lack evidence but not the total count, doesn't
show the repeatable/positional/`N=text` form, and the requirement isn't stated in `--help` or
pre-flight.

## Impact — causes downstream misattribution

A downstream project (teach-me) hit this (compounded by #178's crash) and wrote into its AGENTS.md:
"`tkt close` has a config validation bug that rejects valid tickets — edit frontmatter
`status: done` directly instead." That guidance **bypasses the close gates entirely** (AC checks,
resolution, evidence, atomic push) — the exact opposite of what the evidence-first workflow wants.
A clearer error + help text would have prevented the workaround becoming project policy.

## Proposed fix

1. Error message: name the **count** ("2 of 3 criteria need evidence") AND show the satisfying
   syntax inline (`--evidence <e1> --evidence <e2> …` or `--evidence N=text`).
2. `tkt close --help`: state that `--evidence` is required **once per validation criterion**
   (repeatable), with the positional and `N=text` forms.
3. Optional: a pre-flight hint on `tkt show`/`tkt ready` that a ticket has N criteria needing N
   evidence values at close.

## Acceptance criteria

- [ ] `tkt close` evidence error names the exact number of criteria needing evidence AND shows the positional / `N=text` syntax to satisfy them
- [ ] `tkt close --help` makes explicit that `--evidence` is required once per validation criterion (repeatable)
- [ ] Cross-referenced with #178 (the text-mode writer crash) so both the crash and the UX are addressed

## Notes

- Root of a real downstream cost: teach-me's AGENTS.md close-instruction folklore traces to this
  (+ #178). Fixing both lets that project drop its hand-edit-status workaround.
