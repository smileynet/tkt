---
id: "196"
title: "parent CLI + query/capabilities surface"
status: done
blocked_by: ["192", "195"]
priority: high
validation_criteria:
  - "the --parent flag is accepted on new, batch, and edit (set and clear)"
  - "query emits parent and supports a --parent filter; capabilities documents the field"
  - "Invalid parent (nonexistent id) is rejected at command time with a clear error"
tags: ["cli", "kind:enhancement", "triage:ready-for-agent", "stream:relations"]
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

- [x] The `--parent` flag is accepted on `new`, `batch`, and `edit` (set and clear)
- [x] `query` emits `parent` and supports a `--parent` filter; `capabilities` documents the field
- [x] An invalid parent (nonexistent id) is rejected at command time with a clear error

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `cli`.

Parent storage exists but new/batch/edit/query have no parent interface. Add dependency #195 so new mutations share valid parent rules; retain completed #192 history.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Resolution (2026-11-03)

Parent controls/query/filter/schema implemented; critic caught and verified repair for malformed or mismatched target filenames. Source/tests/evidence promoted; fixtures self-clean, shared target retained for queue. docs/evidence/parent-cli-labels-debug-196-198-237.md.

### Verification
1. ✓ the --parent flag is accepted on new, batch, and edit (set and clear) — "Parent CLI fixtures prove new/batch/edit set and edit clear, including canonical existing parent."
2. ✓ query emits parent and supports a --parent filter; capabilities documents the field — "Query emits parent and filters children; capabilities public fixtures pass."
3. ✓ Invalid parent (nonexistent id) is rejected at command time with a clear error — "Missing/self/cycle/malformed/mismatched targets reject without writes; final gate340Rust42Nodepass one existing skip."
