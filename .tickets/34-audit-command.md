---
id: "34"
title: "tkt audit command: batch closure quality check"
status: open
blocked_by: ["27"]
---

# tkt audit command: batch closure quality check

## What to build

A new `tkt audit` command that scans all tickets for quality issues — the batch-check companion to `tkt validate` (which checks structural integrity). While validate catches malformed tickets, audit catches low-quality closures.

```
tkt audit [--strict] [--brief]
```

### Checks

| Finding | Severity | Meaning |
|---------|----------|---------|
| `unchecked-acs-on-done` | warning | Done ticket with ALL ACs still unchecked |
| `tbd-resolution` | warning | Done ticket with "TBD" or no resolution text |
| `missing-resolution` | warning | Done ticket without a `## Resolution` section |
| `stale-wip` | info | In-progress ticket with file mtime > 7 days |
| `high-priority-open` | info | High-priority ticket still on the frontier |

### Output

Same contract as `tkt validate`:
- Default: JSON `{"status":"pass|fail","findings":[...]}`
- `--brief`: human-readable one-liner per finding
- `--strict`: warnings become errors (exit 1)
- Exit 0 = clean, exit 1 = findings above threshold

### Relationship to validate

`tkt validate` = structural integrity (cycles, dangling deps, bad status values)
`tkt audit` = quality/completeness (did we actually finish the work properly?)

Both can run in CI; validate is a hard gate, audit is a soft quality signal.

## Deletion test

Without this, the only way to find low-quality closures is manual inspection. The 14 unchecked-AC warnings in tkt's own corpus prove the need.

## Acceptance criteria

- [ ] `tkt audit` reports findings on done tickets with quality issues
- [ ] Checks: unchecked-acs, tbd-resolution, missing-resolution, stale-wip, high-priority-open
- [ ] Output format matches tkt validate (JSON default, --brief, --strict)
- [ ] Exit codes: 0=pass, 1=fail (same contract)
- [ ] Integration test with corpus containing known quality issues
