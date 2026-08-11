---
id: "83"
title: "tkt note <id>: open ticket body in $EDITOR"
status: open
blocked_by: []
priority: low
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

# tkt note <id>: open ticket body in $EDITOR

## What to build

TBD

## Acceptance criteria

- [ ] TBD
