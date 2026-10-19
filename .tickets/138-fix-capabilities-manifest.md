---
id: "138"
title: "Update capabilities manifest with all commands"
status: backlog
blocked_by: ["127"]
priority: high
validation_criteria:
  - "tkt capabilities lists all 20+ commands"
---

# Update capabilities manifest with all commands

## Context

`tkt capabilities` is the structured discovery surface for agents, but it currently
advertises only 11 commands while the CLI exposes 21. The omitted commands are
`init`, `doctor`, `migrate`, `context`, `blocked`, `rebase`, `renumber`, `sync-plan`,
`audit`, and `telemetry`. Several represented schemas also drift: creation omits
`tags` and `requires`, while `edit` advertises `status: done` even though users must
close through `tkt close`. Sending an agent from help or init guidance to an
incomplete manifest would make the advertised discovery workflow unreliable.

## What to build

Extend the manifest to cover every public CLI command and its meaningful input fields,
effects, and output behavior. Make the command inventory derived from, or mechanically
compared with, Clap so a public command cannot silently be omitted. Schema values must
describe the accepted user contract rather than merely Rust field types. Coordinate the
agent-facing wording with #127; this implementation remains intentionally outside the
v0.3.1 bugfix release.

## Acceptance criteria

- [ ] `tkt capabilities` includes all public CLI commands, including the ten currently omitted commands
- [ ] Each added mutation command declares its effects and meaningful input schema
- [ ] Existing schemas include `new`/`batch` tags and requires, and never advertise
      `edit --status done` as a supported lifecycle transition
- [ ] A regression test or derived command inventory prevents command and field drift
- [ ] #127's help/init guidance can safely point agents to the complete manifest
