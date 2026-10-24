---
id: "217"
title: "Migrate v0.4 bare skill symlinks before plugin upgrades"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "A dry-run inventories the v0.4 installer symlinks without mutation and execute removes only links proven to target the explicitly supplied old tkt skill directory."
  - "Regular directories, links to other targets, and unrelated user files remain unchanged; second execution is identical and removed/dangling old-source cases are explicit."
  - "Upgrade guidance and isolated tests cover all three former harness roots and bind candidate regeneration to the documentation/runtime change."
tags: ["stream:tooling"]
---

# Migrate v0.4 bare skill symlinks before plugin upgrades

## What to build

TBD

## Acceptance criteria

- [ ] TBD
