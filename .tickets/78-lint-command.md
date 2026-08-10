---
id: "78"
title: "tkt lint: style normalization for cleaner diffs"
status: open
blocked_by: []
---

# tkt lint: style normalization for cleaner diffs

## Problem

Tickets created by different sessions have inconsistent style: unquoted IDs, inconsistent field ordering, trailing whitespace, mixed quoting in blocked_by arrays. This creates noisy diffs when multiple agents touch the same corpus.

## What to build

```bash
tkt lint              # normalize all tickets in-place
tkt lint --check      # report deviations without fixing (CI mode)
tkt lint 05 07        # normalize specific tickets
```

### Normalizations (style only, never semantic)

1. **ID quoting**: `id: 01` → `id: "01"`
2. **blocked_by quoting**: `[01, 04]` → `["01", "04"]`
3. **Field ordering**: id, title, status, blocked_by, priority, env, spec (canonical order)
4. **Trailing whitespace**: removed from frontmatter lines
5. **Consistent blank lines**: one blank after `---` closer, one before body heading
6. **Empty blocked_by**: `blocked_by:[]` → `blocked_by: []` (space after colon)

### What it does NOT do

- Change status, priority, or any semantic field
- Touch the body (user-owned)
- Remove or add fields
- Reformat body markdown

### CI integration

`tkt lint --check` exits 1 if any file would change, 0 if all canonical. Suitable for a pre-commit hook or CI check.

## Acceptance criteria

- [ ] Normalizes quoting style for id and blocked_by
- [ ] Enforces canonical field ordering in frontmatter
- [ ] Removes trailing whitespace from frontmatter
- [ ] `--check` mode for CI (exit 1 on deviation)
- [ ] Never modifies body content
- [ ] Specific ticket IDs can be targeted
- [ ] Idempotent: running twice produces no diff
