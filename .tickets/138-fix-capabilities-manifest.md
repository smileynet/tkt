---
id: "138"
title: "Update capabilities manifest with all commands"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "tkt capabilities lists all 20+ commands"
tags: ["tooling", "kind:bug", "triage:ready-for-agent", "stream:discovery"]
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

- [x] `tkt capabilities` includes all public CLI commands, including all commands currently omitted
- [x] Each added mutation command declares its effects and meaningful input schema
- [x] Existing schemas include `new`/`batch` tags and requires, and never advertise
      `edit --status done` as a supported lifecycle transition
- [x] Each command's inputSchema includes every flag its `--help` documents (e.g. `new`
      includes `--tags`, `--vc`/`--validation`, `--requires` — gaps found by #208)
- [x] A regression test or derived command inventory prevents command and field drift
- [x] #127's help/init guidance can safely point agents to the complete manifest

## Consumer evidence (from #208, consolidated)

A real consumer surfaced the gap: grey-matter's `verify-tkt-usage` skill validates inlined
tkt commands against this manifest and false-flagged valid flags/commands because the
manifest is incomplete at 0.4.0 (92d4833). Specifics it found:

- `new` inputSchema omits `--tags`, `--vc`/`--validation`, `--requires`.
- Command list omits `context`, `blocked`, `audit`, `sync-plan`, `renumber`, `init`,
  `doctor`, `migrate`, `rebase`, `telemetry`, `config`.

An automation that trusts the manifest as authoritative (as its description invites) will
mis-validate until this is fixed — so the single-source (clap-derived) + drift test is
load-bearing, not nice-to-have. (#208 is parked as a duplicate, pending cancellation support in #227.)

## Triage — 2026-11-02

Disposition: `kind:bug`, `triage:ready-for-agent`; stream `tooling`.

Confirmed: capabilities lists 11 commands including config, but omits context/blocked/audit/sync-plan/renumber/init/doctor/migrate/rebase/telemetry. Own command/flag completeness; #208 is its duplicate.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Implementation evidence — 2026-11-02

The manifest now derives its complete command/flag inventory from Clap: 21 product
commands plus built-in help. Domain overlays describe the supported lifecycle,
aliases, numeric/array inputs, optional-value flags, conflicts, mutation modes and
default output formats. Help has a special command-path schema because Clap's
built-in help dispatcher does not parse ordinary subcommand flags.

Fresh public CLI tests compare every command's help flags against the manifest,
and check representative domain values, aliases and modes. Full `mise run check`
passed after correcting one test-only regex-in-loop Clippy failure: 38 Node tests
passed, one existing Windows relative-link fixture skipped; fmt check passed,
Clippy had no warnings, and 307 Rust tests passed. No hosted CI was requested.

Observed gap: edit currently accepts done without close gates. The manifest follows
the supported close-only contract; #241 owns the independently reproduced runtime
fix. #127 still owns the top-level help/init recommendations; the manifest is ready
for those pointers. No runtime mutation behavior was changed by #138.

## Resolution (2026-11-03)

Derived the complete command/flag manifest from Clap, added supported domain schemas, mutation modes and output metadata, and documented the wire format. Full local gate: 307 Rust passes, 38 Node passes and one existing Windows skip; fmt and Clippy clean. Independent Standards and Spec reviews found no actionable issues. Installed source 62a40a3 returned 22 commands. Promoted proof to docs/evidence/capabilities-138.md and the separate lifecycle bypass reproduction to ticket 241; own temporary probe and scripts sent to OS trash. Kept shared maintainer target cache for follow-on 127/153 checks.

### Verification
1. ✓ tkt capabilities lists all 20+ commands — "Pass: installed tkt 0.4.0 (62a40a3), matching implementation HEAD, returned 22 commands; public integration tests compare every help command and flag. Full mise run check passed 307 Rust tests and 38 Node tests, with one existing Windows relative-link skip; fmt no diff and Clippy zero warnings. Independent Standards/Spec reviews passed; durable proof docs/evidence/capabilities-138.md."
