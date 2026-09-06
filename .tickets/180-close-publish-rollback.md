---
id: "180"
title: "close/claim/edit write the ticket file before publish with no rollback on push failure"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "A failed publish (commit/push) after file.write() no longer leaves the ticket mutated-but-unpushed silently; either pre-flight prevents it, the file is restored, or the user gets a clear 'written locally, not pushed' message"
  - "Regression test covers a push failure after write and asserts the resulting state is consistent or clearly surfaced"
tags: ["cli"]
---

# close/claim/edit write the ticket file before publish with no rollback on push failure

## What to build

TBD

## Acceptance criteria

- [ ] TBD
