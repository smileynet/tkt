---
id: "233"
title: "tkt edit can set and clear tags"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "edit --tags replaces tags"
  - "edit --tags '' clears them"
tags: ["cli"]
---

# tkt edit can set and clear tags

## Origin

Field report from philippines-trip, 2026-11-01 (prioritization review, location planning and a Codex worktree takeover). Sam asked for these to be filed.

## What happened

Retagging or adding a secondary tag meant hand-editing frontmatter; `tkt edit` has no `--tags`. #177 only documents the gap.

## Acceptance criteria

- [ ] `tkt edit <id> --tags a --tags b` replaces the tag list; `--tags ''` clears it; `--add-tag` and `--remove-tag` adjust it.
- [ ] Commit message and capabilities reflect the field; tests cover each form.
