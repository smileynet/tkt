---
id: "28"
title: "UX improvements: audit command, close flow polish, output consistency"
status: open
blocked_by: ["27"]
---

# UX improvements: audit command, close flow polish, output consistency

## Context

Research into CLI UX best practices (clig.dev, cargo/gh/git patterns), JTBD analysis of work completion, and audit/doctor command prior art (npm audit, cargo clippy, brew doctor) reveals several areas where tkt can reduce cognitive load and improve the information hierarchy.

The core JTBD for `tkt close` isn't "update a status field" — it's *achieving cognitive closure*. The user wants to free mental bandwidth, unblock dependents, and discover what's next. Every extra step or ambiguous output is friction against that job.

## What to build

### 1. `tkt audit` command — batch closure quality check

Scans all `done` tickets for quality issues. A post-hoc alternative to blocking every close with ceremony.

```
tkt audit [--strict] [--brief]
```

**Checks:**
- Done tickets with all ACs unchecked (never verified)
- Done tickets with "TBD" resolution stubs (never explained)
- Done tickets without a Resolution section at all
- In-progress tickets with no activity (stale WIP — heuristic: mtime > 7 days)
- High-priority tickets still open (attention signal)

**Output:** Same format as `tkt validate` (JSON default, `--brief` for human). Findings have severity: `warning` for informational, `error` for things that should be fixed.

**Exit codes:** 0 = clean, 1 = findings above threshold (respects `--strict`)

### 2. `tkt close` — show newly unblocked tickets

After closing, tell the user what they freed up (Linear/GitHub pattern: "closing this unblocks X"):

```
closed 05-auth-system.md (dated Resolution written)
  acceptance criteria: 3/3 checked ✓
  → unblocked: 06 API endpoints, 07 Deploy pipeline
```

This completes the JTBD cycle: close → confirm → discover next.

### 3. Output consistency — action-result pattern

Standardize all command output to the gh-style action-result pattern:

| Command | Current | Proposed |
|---------|---------|----------|
| `tkt new` | `allocated 01-foo.md (pushed — id claimed, status: open)` | `✓ created 01 foo (pushed)` |
| `tkt claim` | `claimed 01-foo.md (in_progress pushed)` | `✓ claimed 01 foo → in_progress` |
| `tkt close` | `closed 01-foo.md (dated Resolution written)` | `✓ closed 01 foo (Resolution written)` |
| `tkt edit` | `edited 01-foo.md: title, blocked_by` | `✓ edited 01 foo (title, blocked_by)` |

Principles:
- Symbol prefix (✓/✗/⚠) as visual anchor
- Show ID + slug (human-readable) not filename
- One sentence per action
- Suggest next action when state changes unblock work

### 4. `tkt ready` — information hierarchy improvement

Current `ready` output is flat. Improve scannability:

```
Ready (3 tickets):
  01  Set up cargo-dist          [HIGH]
  03  Write API docs
  05  Deploy to staging

In progress (1):
  02  Build auth system          (claimed by session 019fb...)
```

Changes: header with count, indentation, dimmed session info for WIP.

### 5. `--quiet` / `-q` flag for scriptability

Commands that produce confirmation output (`new`, `claim`, `close`, `edit`) should support `-q` to suppress it, printing only the essential datum (ID for new, nothing for others). Enables `tkt new foo --title "X" -q` → outputs just `01`.

## Deletion test

Without these changes, tkt output requires careful reading to extract state; the close workflow doesn't suggest what's next; and there's no way to batch-check closure quality across a project. The tool works but doesn't flow.

## Acceptance criteria

- [ ] `tkt audit` command reports quality issues on done tickets
- [ ] `tkt audit --strict` promotes warnings to errors (exit 1)
- [ ] `tkt close` shows newly unblocked tickets after success
- [ ] Command output uses consistent action-result pattern with symbol prefixes
- [ ] `tkt ready` shows count headers and groups WIP separately
- [ ] `-q` flag suppresses confirmation messages, emits only essential data
- [ ] No breaking changes to `--json` output (machine format unchanged)
- [ ] Integration tests for audit findings and unblocked-after-close

## Design notes (from research)

**Progressive disclosure:** Default output is brief (what changed + what's next). `--verbose` shows paths and timing. `--json` gives full structured data. `--quiet` gives only the datum.

**Stderr vs stdout contract:** Confirmations and diagnostics go to stderr when output is piped (TTY detection). Data (JSON, IDs) always goes to stdout. This enables `tkt new foo -q | xargs tkt claim`.

**Color:** Green ✓ for success, yellow ⚠ for warnings, red ✗ for errors. Respect `NO_COLOR=1` env var. Never encode meaning *only* in color (always pair with symbol).

**Cognitive closure (JTBD):** The close command's real job is freeing mental bandwidth. Show what was completed, confirm nothing was forgotten (AC count), and reveal what's newly available. The user should feel *done* after running close, not uncertain.
