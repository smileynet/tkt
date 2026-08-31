---
id: "177"
title: "guidance: state evidence↔validation_criteria rule, edit-has-no-tags, fill-body-at-new, link-the-source"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "a surface states the evidence↔validation_criteria 1:1-by-index rule plainly (test: manual doc review)"
  - "init.rs snippets instruct fill-body-at-new and link-the-source (test: manual doc review)"
  - "edit's lack of --tags documented as an intentional constraint (test: manual doc review)"
---

# guidance: state evidence↔validation_criteria rule, edit-has-no-tags, fill-body-at-new, link-the-source

## Context

Parent: #172 (doc halves of gaps 1, 2, 3). Review (`.scratch/review/guidance-body.md`)
found the init.rs snippets — the highest-traffic surface, injected into every project —
are silent on all three, and no surface states the evidence↔validation_criteria rule
plainly (the direct cause of the "criterion N does not exist" confusion).

## What to build

Propagate across all guidance surfaces per `.memory/agent-guidance-surfaces.md`
(init.rs snippets, SKILL.md, commands.md, ticket-format.md, ticket-standards.md,
frontier-work.md, AGENTS.md, README):

1. **evidence↔validation_criteria** — state plainly: body `## Acceptance criteria`
   checkboxes ← `--check-all`/`--ac`; frontmatter `validation_criteria` ← `--evidence`
   1:1 by index; the counts are independent.
2. **fill the body at `tkt new`** — don't ship the `TBD` stub (pairs with #174/#176).
3. **link the source** — `--spec`, or a `#NN`/parent reference in the body (pairs with #175).
4. **`tkt edit` has no `--tags`** — intentional; retro-tagging is rare by design.

Best sequenced after #174/#176 land so the docs describe the new behavior.

## Acceptance criteria

- [ ] A surface states the evidence↔validation_criteria 1:1-by-index rule plainly
- [ ] init.rs snippets instruct fill-body-at-new and link-the-source
- [ ] `edit`'s lack of `--tags` documented as an intentional constraint
- [ ] All surfaces synced per the guidance-surfaces checklist; `deploy-skills.sh` run
- [ ] Version bump if init snippets changed (baked into binary)
