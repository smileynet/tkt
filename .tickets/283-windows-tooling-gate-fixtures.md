---
id: "283"
title: "Repair Windows tooling-gate fixture failures (29 failing on clean tree)"
status: in_progress
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

- [ ] Every failing fixture is either repaired or Windows-skipped with a recorded reason
- [ ] `node tools/check-tools.mjs` passes on Windows with zero unexpected failures
- [ ] Fixes verified to still pass (no assertion weakened; checked by reading the diff)
- [ ] Remaining platform-specific gaps are recorded in the ticket or linked docs
