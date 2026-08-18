---
id: "109"
title: "Enhance tkt doctor: flag non-tkt repos, --strict, -o json"
status: in_progress
blocked_by: []
priority: medium
validation_criteria:
  - "tkt doctor ~/code flags non-tkt git repos"
  - "tkt doctor ~/code --strict escalates warnings to errors"
  - "tkt -o json doctor ~/code emits JSON Lines"
  - "cargo test passes"
---

# Enhance tkt doctor: flag non-tkt repos, --strict, -o json

## Problem

`tkt doctor <path>` already scans recursively and validates, but silently skips non-tkt repos (no visibility into adoption gaps), ignores `--strict`, and has no JSON output mode.

## What to build

Three enhancements to the existing cross-project scan:

### 1. Flag non-tkt git repos

In the scan loop, detect git repos without `.tickets/` and report them:

```
  ✓ tkt (12 tickets)
  ✓ recall (38 tickets)
  · some-other-repo (git repo, no .tickets/)
```

Not an error — just visibility. Include in the summary count.

### 2. --strict flag

Pass strict mode into validate checks — escalate warnings to errors in the cross-project scan (currently hardcoded non-strict).

### 3. JSON output

When global `-o json` is active, emit JSON Lines per project:

```json
{"path":"/Users/x/code/tkt","status":"pass","tickets":12,"errors":0,"warnings":0}
{"path":"/Users/x/code/recall","status":"pass","tickets":38,"errors":0,"warnings":2}
{"path":"/Users/x/code/other","status":"no_tickets","is_git":true}
```

## Context

- `src/commands/doctor.rs` — `run_cross_project()` at line ~130, `find_ticket_dirs_recursive()` at line ~215
- Already has: recursive discovery, corpus loading, validate checks, summary output
- Global `-o json` flag already exists (from ticket 85)

## Acceptance criteria

- [ ] Non-tkt git repos listed with "no .tickets/" indicator
- [ ] Summary includes non-tkt count
- [ ] `--strict` escalates warnings to errors in cross-project mode
- [ ] `-o json` emits JSON Lines per project (pass/fail/no_tickets)
- [ ] Exit code: 0 all pass, 1 any errors
- [ ] Existing single-project doctor behavior unchanged
