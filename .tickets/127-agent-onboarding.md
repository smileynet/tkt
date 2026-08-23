---
id: "127"
title: "Improve agent discoverability: help nudge to capabilities, init recommends agent setup"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "tkt --help includes a note pointing agents to tkt capabilities for structured discovery"
  - "tkt init output recommends adding skill and AGENTS.md guidance for maximum effectiveness"
---

# Improve agent discoverability: help nudge to capabilities, init recommends agent setup

## Problem

Agents encountering tkt for the first time have no clear path to discover all available commands and their schemas. `tkt --help` gives a flat list with one-line descriptions. `tkt capabilities` gives structured JSON with schemas — but nothing points agents there. `tkt init` sets up the directory and prints workflow basics, but doesn't tell agents what AGENTS.md content or skills to install for maximum effectiveness.

The result: agents use a fraction of tkt's features because they never discover the full surface.

## What to build

### 1. Help nudge to capabilities

Add a line to `tkt --help` output (after the command list) that tells agents where to get structured discovery:

```
For machine-readable command schemas: tkt capabilities
```

This is the equivalent of "run `tkt capabilities` to understand what I can do" — agents parsing help output will see this and know to call it.

### 2. Init recommends agent setup

When `tkt init` runs, after creating `.tickets/` and printing the workflow snippet, add a recommendation section:

```
Agent setup recommendations:
  • Add the tkt section to your AGENTS.md: tkt init --write
  • For deeper agent integration, install the tkt skill:
    cargo install tkt  (includes skills/tkt/ for kiro/claude/codex)
  • Run tkt capabilities for machine-readable command schemas
```

This gives agents (and humans setting up for agents) a clear "what else should I do" path.

### 3. Update capabilities manifest (prerequisite)

`tkt capabilities` currently only lists 11 of ~20 commands. It needs to include: context, migrate, audit, blocked, doctor, init, rebase, renumber, sync-plan, telemetry. Otherwise the nudge sends agents to an incomplete manifest.

## Context

- **Relevant files:** `src/cli.rs` (clap about/after_help), `src/commands/capabilities.rs` (manifest), `src/commands/init.rs` (output)
- **Clap mechanism:** `#[command(after_help = "...")]` adds text after the auto-generated help

## Acceptance criteria

- [ ] `tkt --help` includes a line pointing to `tkt capabilities`
- [ ] `tkt init` output includes agent setup recommendations
- [ ] `tkt capabilities` lists ALL commands (not just the current subset)
- [ ] Agents following the init recommendations get full tool effectiveness
