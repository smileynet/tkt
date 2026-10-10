---
id: "195"
title: "parent validation: dangling + cycle checks"
status: open
blocked_by: ["192"]
priority: high
validation_criteria:
  - "check_dangling_parent flags a parent id with no matching ticket"
  - "check_parent_cycles detects a parent chain cycle; a ticket cannot be its own parent"
  - "Both checks wired into collect_findings and covered by tests mirroring check_cycles"
---

# parent validation: dangling + cycle checks

## What to build

TBD

## Acceptance criteria

- [ ] TBD
