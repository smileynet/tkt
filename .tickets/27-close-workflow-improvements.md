---
id: "27"
title: "Recommend: tkt close should warn on unchecked ACs and require resolution text"
status: done
blocked_by: []
---

# tkt close workflow improvements

## Context

During godot-helper development (16 tickets closed in one session), codex review found:
- 14 of 16 closed tickets still had unchecked acceptance criteria
- 8 had `Resolution: TBD` stubs (the default appended by `tkt close`)
- Tickets were sometimes closed in a commit before the implementation commit (due to commit ordering)

The current `tkt close` behavior appends a `## Resolution` stub with "TBD" and warns about unchecked ACs, but proceeds anyway. The warning is easy to miss in a fast workflow.

## Recommendations

### 1. Make resolution text required (or strongly prompted)

When `tkt close <id>` runs:
- If all ACs are unchecked: **error** — refuse to close (must check at least one, or use `--force`)
- If resolution is "TBD" or empty: **prompt** for resolution text inline, or accept `--resolution "text"`

```
tkt close 05 --resolution "Added format:watch task with sources/exclusions"
```

### 2. Add `--check-ac` flags to mark ACs done at close time

```
tkt close 05 --check 1,2,3 --resolution "Implemented watch mode"
```

### 3. Batch resolution backfill command

```
tkt backfill-resolutions   # Interactive: walks through done tickets with TBD resolutions
```

### 4. CI pattern recommendation

For automated/agent workflows where tickets close rapidly:
- `tkt close` could accept `--resolution` as required arg (configurable via `.tickets/config.toml`)
- Or: a post-session `tkt audit` command that reports incomplete closures

## Acceptance criteria

- [x] `tkt close` errors on all-unchecked ACs (or requires --force)
- [x] `tkt close --resolution` flag accepted
- [x] Unchecked AC count is prominent in close output (not just a warning)
- [x] Consider: `tkt audit` command for batch checking closure quality
