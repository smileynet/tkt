---
id: "157"
title: "tkt hooks subcommand + pre-commit installer (warn-default, config block/log)"
status: backlog
blocked_by: []
priority: medium
validation_criteria:
  - "tkt hooks install writes a worktree-aware pre-commit shim that warns on staged status:done without Resolution"
  - "block and logging are opt-in via [hooks] config (both off by default); existing hooks chained via .legacy backup"
  - "Windows: LF-only shim, zero new deps, lossless tkt hooks uninstall"
tags: ["contract"]
---

# tkt hooks subcommand + pre-commit installer (warn-default, config block/log)

## What to build

TBD

## Acceptance criteria

- [ ] TBD
