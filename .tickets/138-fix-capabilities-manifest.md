---
id: "138"
title: "Update capabilities manifest with all commands"
status: open
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
`audit`, and `telemetry`. Sending an agent from help or init guidance to an incomplete
manifest would make the advertised discovery workflow unreliable.

## What to build

Extend the manifest to cover every public CLI command and its meaningful input fields,
effects, and output behavior. Add a regression test or command-set comparison so a new
Clap command cannot silently be omitted. Coordinate the agent-facing wording with #127;
this implementation remains intentionally outside the v0.3.1 bugfix release.

## Acceptance criteria

- [ ] `tkt capabilities` includes all public CLI commands, including the ten currently omitted commands
- [ ] Each added mutation command declares its effects and meaningful input schema
- [ ] A regression test or derived command inventory prevents manifest drift
- [ ] #127's help/init guidance can safely point agents to the complete manifest
