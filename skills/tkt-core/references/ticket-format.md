# Ticket File Format

## Location

`.tickets/{NN}-{slug}.md` — one file per ticket, numbered for ordering.

## Full Template

```markdown
---
id: "NN"
title: "Short behavioral title"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "criterion 1"
  - "criterion 2"
tags:
  - "backend"
  - "api"
requires:
  - "corp"
---

# {Title}

## What to build

End-to-end behavior from the user's perspective — NOT implementation steps.
Describe WHAT the system should do, not HOW to build it.
Avoid file paths and line numbers (they go stale). Reference interfaces, types, contracts.

## Context

- **Relevant files:** {paths the implementer should read first}
- **Relevant decisions:** {ADR or spec sections that inform this work}
- **Domain terms:** {any non-obvious vocabulary, link to CONTEXT.md}

## Acceptance criteria

- [ ] Criterion 1 (concrete, testable, independently verifiable)
- [ ] Criterion 2
- [ ] Criterion 3

## Research / Spikes (if applicable)

- **Research:** {question to answer} — method: {web search / codebase / docs}
- **Spike:** {hypothesis to prove} — time-box: {hours} — pass/fail: {criteria}

## Out of scope

- {What this ticket does NOT include}
- {Adjacent work that should be a separate ticket}
```

## Required Fields

| Field | Type | Notes |
|-------|------|-------|
| `id` | quoted string | Must match filename prefix: `"07"` → `07-slug.md` |
| `title` | quoted string | Human-readable, appears in `tkt ready` output |
| `status` | enum | `open \| backlog \| in_progress \| done` (default: `open`) |
| `blocked_by` | array of quoted strings | `["01", "03"]` — IDs that must be `done` first |

## Optional Fields

| Field | Type | Notes |
|-------|------|-------|
| `priority` | enum | `urgent > high > medium > low` (default: medium) |
| `env` | enum | `corp \| personal \| either` — filters `tkt ready` via `CREW_ENV` |
| `spec` | string | Links to originating spec slug |
| `validation_criteria` | string array | Machine-checkable criteria for agent close gate |
| `tags` | string array | Work-stream / categorization tags — the primary scoping mechanism for `tkt ready` and `tkt context`. Set via `--tags` at creation (recommended) or auto-applied from `tkt context`. |
| `requires` | string array | Machine capabilities required (e.g., `gpu`, `linux`, `corp`). Ticket only appears in `tkt ready` if `machine.capabilities` config includes all listed values. Backward compat: if `env` is set and `requires` is empty, requires is synthesized from env. |

## Status Lifecycle

| Status | Meaning | Enters when |
|--------|---------|-------------|
| `backlog` | Parked, excluded from frontier | Deliberately deferred out of the current cycle |
| `open` | Available for work (the default) | Created and unblocked |
| `in_progress` | Currently being worked | `tkt claim <id>` |
| `done` | Completed, verified | `tkt close <id>` — all ACs must be checked |

## Formatting Rules

- IDs are always **quoted strings**: `id: "07"`, not `id: 7`
- blocked_by values are always **quoted**: `blocked_by: ["01", "03"]`
- Filename prefix matches id exactly (no zero-padding mismatch)
- Acceptance criteria use `- [ ]` checkbox syntax

## Principles

- **Default to `open`, not `backlog`** — new work is frontier-eligible unless deliberately deferred out of this cycle; parking actionable work hides it
- **Tag at creation** — set `tags` (via `--tags`) when the ticket is born in multi-stream projects; retro-tagging rarely happens
- **Behavioral, not procedural** — "User can export CSV" not "add CSV renderer to ExportService"
- **Durable over precise** — interfaces and contracts, not file paths and line numbers
- **One concern per ticket** — if title needs "and", split
- **Acceptance criteria are the contract** — work is done when these pass, nothing more
- **ACs must be checked before closure** — `tkt close` enforces this by default (`--force` to override)
