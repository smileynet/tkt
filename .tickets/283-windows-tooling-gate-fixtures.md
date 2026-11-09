---
id: "283"
title: "Repair Windows tooling-gate fixture failures (29 failing on clean tree)"
status: done
blocked_by: []
validation_criteria:
  - "every failing fixture repaired or Windows-skipped with recorded reason"
  - "node tools/check-tools.mjs passes on Windows with zero unexpected failures"
  - "no assertion weakened; verified by reading the diff"
  - "remaining platform gaps recorded"
tags: ["kind:bug"]
---
## What to build

Characterize and repair the Windows tooling-gate fixture failures so
`mise run check` / `node tools/check-tools.mjs` can go green on this Windows
machine.

**Observed 2026-11-08:** 29 failing assertions on a clean tree (verified by
stashing all local changes and re-running), in these families: freshness
wrapper (3), explicit-base scope (1), CI seal (3), CI candidate identity (1),
static manifest checks (1), SessionStart hook (2), legacy migration (1), plus
temp-dir cleanup `rm` EPERM. Likely shares root causes with grey-matter
ticket 213 (Windows symlinkSync EPERM in gate fixtures).

**Approach:** group the failures by root cause (symlink privileges, temp-dir
cleanup, path/CRLF handling, process spawn), fix each family or skip it on
Windows with a recorded justification, and note remaining platform evidence
gaps. Do not weaken what a fixture asserts to make it pass.

## Acceptance criteria

- [x] Every failing fixture is either repaired or Windows-skipped with a recorded reason
- [x] `node tools/check-tools.mjs` passes on Windows with zero unexpected failures
- [x] Fixes verified to still pass (no assertion weakened; checked by reading the diff)
- [x] Remaining platform-specific gaps are recorded in the ticket or linked docs

## Resolution (2026-11-09)

done: three root causes fixed in fixture plumbing (read-only git objects + dangling junctions broke rmSync cleanup -> shared removeTree helper across 13 test files; bare C:\ path to node --import -> file:// URL; mise entry-point case now skips with a diagnostic when mise is absent). Two remaining gaps are environment, not code: gates need a real Node 22+ (the Kimi session's Electron-as-Node shim fails spawn fixtures with an ICU fd error) and mise on PATH for the optional mise entry-point case

### Verification
1. ✓ every failing fixture repaired or Windows-skipped with recorded reason — "rmSync cleanup failures (EPERM on 444 git objects, ERROR_CANT_RESOLVE_FILENAME on dangling junctions) fixed by tools/test-fixtures.mjs removeTree; verified at 18 sites across 13 files"
2. ✓ node tools/check-tools.mjs passes on Windows with zero unexpected failures — "node tools/check-tools.mjs: 50 pass, 0 fail, 1 intentional skip (pre-existing relative-link fixture), run with real Node 22.23.1 + cargo on PATH on this Windows machine"
3. ✓ no assertion weakened; verified by reading the diff — "diff touches only fixture cleanup, an --import URL conversion, and an availability-gated mise case; every assertion unchanged (readable in commit 4946bbf)"
4. ✓ remaining platform gaps recorded — "gaps recorded: Electron-as-Node runtime incompatible with spawn fixtures (added to grey-matter windows gotchas); mise-on-PATH precondition now self-skipping with diagnostic"
