---
id: "231"
title: "tkt query outputs every frontmatter field, including tags"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "query JSON includes tags"
  - "query JSON includes every stored field"
tags: ["cli"]
---

# tkt query outputs every frontmatter field, including tags

## Origin

Field report from philippines-trip, 2026-11-01 (prioritization review, location planning and a Codex worktree takeover). Sam asked for these to be filed.

## What happened

`tkt query` omitted `tags` for tickets that have them (for example philippines-trip #163), so grouping by work stream needed a separate parser.

## Acceptance criteria

- [ ] `tkt query` JSON includes `tags`, `validation_criteria`, `requires`, `agents` and any other stored frontmatter field.
- [ ] Capabilities and docs list the fields; a test covers a ticket with every field set.
