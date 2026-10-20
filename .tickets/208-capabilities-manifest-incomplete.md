---
id: "208"
title: "tkt capabilities -o json is incomplete: omits flags (--tags, --vc, --requires) and commands (context, blocked, audit, ...)"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "tkt capabilities -o json lists every top-level command that tkt --help lists (context, blocked, audit, sync-plan, renumber, init, doctor, migrate, rebase, telemetry, config included)"
  - "Each command's inputSchema includes every flag its --help documents (e.g. new includes --tags, --vc/--validation, --requires)"
  - "A test asserts capabilities output stays in sync with the actual clap command/flag definitions so it can't drift again"
tags: ["type:bug", "stream:tooling"]
---

# tkt capabilities -o json is incomplete: omits flags (--tags, --vc, --requires) and commands (context, blocked, audit, ...)

## What to build

TBD

## Acceptance criteria

- [ ] TBD
