---
id: "41"
title: "evaluate claim usage: is the claim step needed for single-agent workflows?"
status: done
blocked_by: []
---

# Evaluate claim usage: is the claim step needed for single-agent workflows?

## Observed

Telemetry shows `claim` used only 2 times across 251 events (both in crew-research). Agents go directly from `tkt ready` to working on the ticket, then `tkt close`. The claim step adds a git push round-trip (~1.9s) that provides no value when only one agent works a repo.

## Current workflow (3 pushes)

```
tkt ready        → pick ticket
tkt claim 05     → push (1.9s) — marks in_progress
  ... work ...
tkt close 05     → push (2.3s) — marks done
```

## Observed workflow (2 pushes)

```
tkt ready        → pick ticket
  ... work ...
tkt close 05     → push (2.3s) — marks done
```

Agents skip claim entirely. The 2 observed claims were both in crew-research (multi-session project where another session might pick the same ticket).

## Options

1. **Keep as-is** — claim is valuable for multi-agent/multi-session repos. Agents that don't need it just skip it.
2. **Make close work on open tickets directly** — currently close accepts open OR in_progress tickets, so skipping claim already works.
3. **Add steering guidance** — document that claim is optional for single-agent repos but recommended for shared repos.
4. **Auto-claim on close** — if status is open when closing, auto-transition through in_progress (no extra push).

## Recommendation

Option 3 (document) — the telemetry confirms the tool already supports skipping claim. Just make it explicit in guidance so agents don't feel they're doing something wrong.

## Acceptance criteria

- [x] Decision documented (keep/change/document)
- [x] If documenting: update AGENTS.md and frontier-work steering
- [x] If changing behavior: implementation + tests

## Resolution (2026-08-07)

Decision: Option 3 — document that claim is optional. Verified: `tkt close` already accepts `open` tickets directly (only rejects `done`). Added note to README Lifecycle section. No code change needed — the behavior was already correct, just undocumented.
