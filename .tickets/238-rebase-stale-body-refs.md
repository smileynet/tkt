---
id: "238"
title: "Renumbering leaves stale ID references in ticket bodies"
status: open
blocked_by: []
validation_criteria:
  - "tkt rebase reports, or rewrites, body references to renumbered IDs"
  - "tkt validate warns on #NNN / ticket NNN references to IDs that do not exist"
---

# Renumbering leaves stale ID references in ticket bodies

## Problem

`tkt rebase` renumbered grey-matter 137→152 and 138→153, but prose references in ticket bodies
("see 138 item 7", "Companion to 137") kept the old numbers and had to be found by grep and fixed
by hand.

## Context

Found 2026-11-01 in a philippines-trip retro. Related done work: #134 (renumber birth-window
citation scan), which covers manual `renumber` but not the collision-recovery rebase path.

## Acceptance criteria

- [ ] `tkt rebase` reports, or rewrites, body references to renumbered IDs.
- [ ] `tkt validate` warns on `#NNN` / `ticket NNN` references to IDs that do not exist.
