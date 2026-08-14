---
id: "78"
title: "tkt lint: style normalization for cleaner diffs"
status: in_progress
blocked_by: []
validation_criteria:
  - "tkt lint normalizes all tickets and git diff shows only style changes"
  - "tkt lint --check exits 0 after tkt lint (idempotent)"
  - "tkt lint --check exits 1 on a deliberately malformed ticket"
  - "body content unchanged after lint (diff only in frontmatter)"
  - "cargo test passes"
---

# tkt lint: style normalization for cleaner diffs

## Problem

Tickets created by different sessions have inconsistent style: unquoted IDs, inconsistent field ordering, trailing whitespace, mixed quoting in blocked_by arrays. This creates noisy diffs when multiple agents touch the same corpus. We just manually fixed this across 9 repos — a lint command prevents it from recurring.

## What to build

```bash
tkt lint              # normalize all tickets in-place
tkt lint --check      # report deviations without fixing (CI mode)
tkt lint 05 07        # normalize specific tickets
```

### Normalizations (style only, never semantic)

| Rule | Before | After |
|------|--------|-------|
| ID quoting | `id: 01` | `id: "01"` |
| blocked_by quoting | `[01, 04]` | `["01", "04"]` |
| Field ordering | random | id, title, status, blocked_by, priority, env, spec, validation_criteria |
| Trailing whitespace | `title: "Foo"  ` | `title: "Foo"` |
| Colon spacing | `blocked_by:[]` | `blocked_by: []` |
| Blank lines | inconsistent | one blank after closing `---`, one before body H1 |

### What it does NOT do

- Change status, priority, or any semantic field value
- Touch the body (user-owned content below frontmatter)
- Remove or add fields (preserves unknown fields in their relative position)
- Reformat body markdown

## Implementation

### Approach: rewrite frontmatter, preserve body

1. Parse ticket with `TicketFile::parse()` (already preserves field order in `fm: Vec<(String, String)>`)
2. Reorder `fm` entries to canonical order (known fields first, unknown fields after in original order)
3. Normalize each value:
   - `id`: ensure quoted (`"01"`)
   - `blocked_by`: ensure `["01", "04"]` format
   - All values: trim trailing whitespace
   - Ensure space after colon (` value` not `value`)
4. Write back: `---\n{fields}\n---\n{body}` with consistent blank line handling
5. Compare old content to new — if different, write (or report in `--check` mode)

### Files to touch

- `src/commands/lint.rs` (new) — command logic
- `src/commands/mod.rs` — register the command
- `src/cli.rs` — add `Lint` subcommand with `--check` flag and optional IDs
- `src/core/ticket.rs` — may need a `TicketFile::render()` method that produces canonical output

### Canonical field order

```
id, title, status, blocked_by, priority, env, spec, validation_criteria
```

Unknown fields (e.g., `estimate`, `type`, `lane`) go after the known fields, in their original relative order.

## Context

- `src/core/ticket.rs` — `TicketFile` struct with `fm: Vec<(String, String)>` preserves raw field order
- `src/commands/close.rs` — example of writing back to a ticket file (via `MutationContext`)
- The manual fix we did today: `sed` to quote IDs and `perl` to quote blocked_by — that's exactly what this command automates

## Acceptance criteria

- [ ] Normalizes quoting style for id and blocked_by
- [ ] Enforces canonical field ordering in frontmatter
- [ ] Removes trailing whitespace from frontmatter
- [ ] `--check` mode for CI (exit 1 on deviation, lists files that would change)
- [ ] Never modifies body content
- [ ] Specific ticket IDs can be targeted (positional args)
- [ ] Idempotent: running twice produces no diff
- [ ] Preserves unknown fields (doesn't drop custom frontmatter keys)
- [ ] Reports count: "N files normalized" or "N files would change"

## Out of scope

- Reformatting body markdown (that's a separate concern)
- Enforcing field presence (that's `tkt validate`)
- Auto-fix of semantic errors like bad status values (that's `tkt validate --fix`)
