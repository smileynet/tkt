---
id: "194"
title: "assignee field + claim-by-assignment + frontier honors it"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "Ticket struct has assignee: Option<String>, parsed/serialized like spec"
  - "tkt claim sets assignee alongside in_progress; refuses an already-assigned ticket without --force"
  - "tkt ready excludes tickets assigned to another identity; query/ready surface assignee"
---

# assignee field + claim-by-assignment + frontier honors it

## What to build

TBD

## Acceptance criteria

- [ ] TBD
