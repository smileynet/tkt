---
id: "138"
title: "Update capabilities manifest with all commands"
status: open
blocked_by: []
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
- [ ] Each command's inputSchema includes every flag its `--help` documents (e.g. `new`
      includes `--tags`, `--vc`/`--validation`, `--requires` — gaps found by #208)
- [ ] A regression test or derived command inventory prevents command and field drift
- [ ] #127's help/init guidance can safely point agents to the complete manifest

## Consumer evidence (from #208, consolidated)

A real consumer surfaced the gap: grey-matter's `verify-tkt-usage` skill validates inlined
tkt commands against this manifest and false-flagged valid flags/commands because the
manifest is incomplete at 0.4.0 (92d4833). Specifics it found:

- `new` inputSchema omits `--tags`, `--vc`/`--validation`, `--requires`.
- Command list omits `context`, `blocked`, `audit`, `sync-plan`, `renumber`, `init`,
  `doctor`, `migrate`, `rebase`, `telemetry`, `config`.

An automation that trusts the manifest as authoritative (as its description invites) will
mis-validate until this is fixed — so the single-source (clap-derived) + drift test is
load-bearing, not nice-to-have. (#208 closed as duplicate of this ticket.)
