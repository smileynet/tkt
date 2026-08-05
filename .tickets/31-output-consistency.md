---
id: "31"
title: "output consistency: action-result pattern with symbol prefixes"
status: done
blocked_by: ["27"]
---

# Output consistency: action-result pattern with symbol prefixes

## What to build

Standardize all mutation command output to a consistent format inspired by the gh CLI's action-result pattern. One symbol, one verb, one line.

### Current → Proposed

| Command | Current | Proposed |
|---------|---------|----------|
| `tkt new` | `allocated 01-foo.md (pushed — id claimed, status: open)` | `✓ created 01 foo (pushed)` |
| `tkt claim` | `claimed 01-foo.md (in_progress pushed)` | `✓ claimed 01 foo → in_progress` |
| `tkt close` | `closed 01-foo.md (dated Resolution written)` | `✓ closed 01 foo (Resolution written)` |
| `tkt edit` | `edited 01-foo.md: title, blocked_by` | `✓ edited 01 foo (title, blocked_by)` |
| `tkt renumber` | `renumbered 01 → 01-new.md (2 inbound ref(s) updated)` | `✓ renumbered 01 → 02 (2 refs updated)` |
| domain error | `tkt: 01 is already done` | `✗ 01 is already done` |

### Principles

- **✓** (green) for success, **✗** (red) for domain errors, **⚠** (yellow) for warnings
- Show ID + slug (human-readable), not raw filename
- One sentence per action
- Respect `NO_COLOR=1` — degrade to plain `✓`/`✗`/`⚠` without ANSI codes
- Machine output (`--json`) unchanged

### Scope

Only mutation commands (new, batch, claim, close, edit, renumber) and error output. Read commands (ready, query, validate) keep their existing formats.

## Deletion test

Without consistency, each command has its own ad-hoc format (some show filenames, some show IDs, some show both). Users must parse different shapes mentally per command.

## Acceptance criteria

- [x] new/batch/claim/close/edit/renumber use `✓ verb ID slug (detail)` format
- [x] Domain errors use `✗ message` format
- [ ] Symbols degrade gracefully when NO_COLOR=1 is set
- [x] No breaking changes to --json output
- [x] Existing integration tests updated to match new format

## Resolution (2026-08-05)

All mutation commands use `success_msg()` with `✓ verb ID slug (detail)` pattern. Domain errors emit `✗ message` to stderr. Tests updated for new output strings. NO_COLOR handling deferred — #47 tracks the color/symbol spec decision (plain UTF-8 glyphs are emitted with no ANSI codes, so NO_COLOR is vacuously respected but the colored behavior from the spec was never implemented).
