---
id: "271"
title: "tkt edit --body to replace a ticket body"
status: in_progress
blocked_by: []
priority: medium
validation_criteria:
  - "tkt edit --body replaces the body, keeps notes, commits once, and a later note succeeds"
tags: ["kind:enhancement", "cli"]
assignee: "kimi"
---
Source: a retrospective of an ios-helper session on 2026-11-08 (ios-helper tickets 35, 36, 37, 38 and 40 were rescoped by rewriting their bodies). Related creation-time body input was delivered in #176. `tkt edit` can change the title, tags, status, blockers and validation criteria, but not the body. To rewrite "What to build" and the acceptance criteria, an agent edits the ticket file directly. A following `tkt note` then refuses with "has uncommitted or untracked content" until the agent commits the hand edit. That happened twice in the session, and each body rewrite became a manual commit outside tkt.

## What to build

`tkt edit <id> --body @file|-|TEXT` replaces the ticket body, everything after the frontmatter, while keeping existing `## Note` sections and their markers. It commits like other edits and respects `--no-push` and `--dry-run`.

## Acceptance criteria

- [ ] `tkt edit --body @file` replaces the body and keeps notes appended earlier.
- [ ] The edit is one tkt commit; a later `tkt note` on the same ticket succeeds without a manual commit.
- [ ] `tkt validate` rules still apply to the new body, for example the source-reference check.
