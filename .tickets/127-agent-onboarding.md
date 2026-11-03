---
id: "127"
title: "Improve agent discoverability: help nudge to capabilities, init recommends agent setup"
status: done
blocked_by: ["138"]
priority: high
validation_criteria:
  - "tkt --help includes a note pointing agents to tkt capabilities for structured discovery"
  - "tkt init output recommends adding skill and AGENTS.md guidance for maximum effectiveness"
tags: ["guidance", "kind:enhancement", "triage:ready-for-agent", "stream:discovery"]
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
    Install the tkt plugin or workflow skills separately; cargo install installs only the CLI
  • Run tkt capabilities for machine-readable command schemas
```

This gives agents (and humans setting up for agents) a clear "what else should I do" path.

> Manifest completeness is owned by #138 (a prerequisite — this ticket is
> `blocked_by: 138`). The nudge here must only point at a manifest that #138 has
> already made complete.

## Context

- **Relevant files:** `src/cli.rs` (clap about/after_help), `src/commands/capabilities.rs` (manifest), `src/commands/init.rs` (output)
- **Clap mechanism:** `#[command(after_help = "...")]` adds text after the auto-generated help

## Acceptance criteria

- [x] `tkt --help` includes a line pointing to `tkt capabilities`
- [x] `tkt init` output includes agent setup recommendations
- [x] Every generated instruction target points to the complete manifest; setup recommendations distinguish CLI installation from workflow skills and preserve snippet-only/quiet output contracts.

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `guidance`.

Keep real dependency #138. Replace the cargo-install-includes-skills example: Cargo.toml currently excludes skills. Describe plugin installation separately from the binary.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Implementation evidence — 2026-11-02

Top-level help now points to `tkt capabilities -o json`. Normal initialization
prints project-instruction, separate plugin/skill and schema-discovery guidance.
All five embedded snippet variants (six generated instruction files, with shared
AGENTS content for Copilot/OpenCode/Codex) retain the discovery pointer. Raw
`--agent-only` output and quiet writes do not gain setup chatter.

Four new public CLI tests began red, then passed for help, default init, every
generated target and quiet/snippet behavior. The final `mise run check` passed
311 Rust tests, fmt and zero-warning Clippy; Node fixtures passed 38 with one
existing Windows relative-link skip. Git fixture line-ending notices were
informational; no global Git configuration was changed. Versions are unchanged;
the changed snippets reach this machine through local install and other users
through the coordinated release path, not a hand-edited version bump.

## Resolution (2026-11-03)

Added the root help discovery pointer and init setup recommendations, synchronized all embedded targets and guidance, and preserved quiet/snippet-only behavior. Full local gate passed 311 Rust tests and 38 Node fixtures, with one existing Windows skip; fmt and Clippy clean. Independent Standards/Spec reviews found no actionable issues. Installed 30e3182 help and init outputs were observed in a local fixture; native Claude checks passed. Promoted proof to docs/evidence/onboarding-127.md; own fixture/scripts/log/messages sent to OS trash; shared maintainer target cache retained for continued development and task153.

### Verification
1. ✓ tkt --help includes a note pointing agents to tkt capabilities for structured discovery — "Pass: installed tkt 0.4.0 (30e3182), matching implementation HEAD, printed the top-level help pointer to tkt capabilities -o json. Public help integration test passed; full local gate passed 311 Rust tests."
2. ✓ tkt init output recommends adding skill and AGENTS.md guidance for maximum effectiveness — "Pass: installed init printed project instructions via tkt init --write, separate workflow skill/plugin setup, CLI-only installation distinction and structured discovery. Public tests passed for default init, all six generated files, raw snippets and quiet writes. Native plugin checks passed; durable proof docs/evidence/onboarding-127.md."
