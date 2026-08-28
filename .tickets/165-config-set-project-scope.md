---
id: "165"
title: "config --set: support project scope (.tickets/config.toml), not just user"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "tkt config --set --project key=value writes .tickets/config.toml (test: config::set_project_scope)"
  - "tkt config --set without flag keeps current user-level behavior (back-compat)"
  - "help text documents both scopes"
tags: ["dx"]
---

# config --set: support project scope (.tickets/config.toml), not just user

## What to build

TBD

## Acceptance criteria

- [ ] TBD
