---
id: "196"
title: "parent CLI + query/capabilities surface"
status: backlog
blocked_by: ["192"]
priority: high
validation_criteria:
  - "the --parent flag is accepted on new, batch, and edit (set and clear)"
  - "query emits parent and supports a --parent filter; capabilities documents the field"
  - "Invalid parent (nonexistent id) is rejected at command time with a clear error"
---

# parent CLI + query/capabilities surface

## What to build

Expose the `parent` field (ticket 192) through the CLI and machine surfaces.

- `src/cli.rs`: add `--parent <id>` to `new`, `batch`, and `edit` (set and
  clear), mirroring `--spec`/`--blocked-by`.
- `src/commands/{new,batch,edit}.rs`: thread the flag through.
- `src/commands/query.rs`: emit `parent` in JSON Lines and support a `--parent`
  filter.
- `src/commands/capabilities.rs`: document the field in the manifest.
- Reject a `--parent` pointing at a nonexistent id at command time.

## Acceptance criteria

- [ ] The `--parent` flag is accepted on `new`, `batch`, and `edit` (set and clear)
- [ ] `query` emits `parent` and supports a `--parent` filter; `capabilities` documents the field
- [ ] An invalid parent (nonexistent id) is rejected at command time with a clear error
