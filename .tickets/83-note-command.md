---
id: "83"
title: "tkt note <id>: open ticket body in $EDITOR"
status: backlog
blocked_by: []
priority: low
validation_criteria:
  - "`tkt note <id>` opens the ticket file in $EDITOR"
tags: ["cli", "kind:enhancement", "triage:needs-info", "stream:workflow", "gate:decision"]
---

# tkt note: open ticket body in $EDITOR

## Context

dstask has a `note` command that opens the task's markdown body in $EDITOR. Currently tkt users need to know the file path (`.tickets/{id}-{slug}.md`) to edit the body. A `tkt note <id>` command removes that friction.

## What to build

```bash
tkt note 03
# → opens .tickets/03-deploy-pipeline.md in $EDITOR, cursor after frontmatter
```

Behavior:
- Resolves ID to filename
- Opens `$EDITOR` (falls back to `$VISUAL`, then `vi`)
- Positions cursor after the `---` frontmatter delimiter if editor supports it
- No git commit on save (body edits are user-owned, not tkt-managed)
- Error if ticket not found

## Acceptance criteria

- [ ] `tkt note <id>` opens the ticket file in $EDITOR
- [ ] Falls back through $EDITOR → $VISUAL → vi
- [ ] Error message if ticket ID doesn't exist
- [ ] Does not commit changes (body is user-owned)
- [ ] Works with numeric ID (finds the matching file)


## Field report — 2026-11-01

Agents can't use an editor. Proposed non-interactive form: `tkt note <id> --append "text"` (or `--file`) adds a dated `## Note — YYYY-MM-DD` section and commits it. In one philippines-trip session, ticket bodies were hand-edited about 15 times for decisions and status notes.

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `cli`.

Deferred proposal combines a noncommitting editor with a committing append form. Decide the mutation/commit contract and noninteractive input before implementation.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Choose the commit contract for interactive editing versus noninteractive append before designing the note interface.
