---
id: "90"
title: "tkt open <id>: find URLs in ticket body and open in browser"
status: open
blocked_by: []
priority: low
validation_criteria:
  - "`tkt open <id>` opens first URL found in ticket body"
tags: ["cli", "kind:enhancement", "triage:ready-for-agent", "stream:workflow"]
---

# tkt open: find URLs in ticket body and open in browser

## Current executable contract — 2026-11-03

Select the first HTTP(S) URL in body order; never launch other schemes. Provide a no-launch listing/preview path and obey global dry-run. Do not prompt interactively for multiple URLs in the first version. Verify exact portable launcher arguments with isolated substitutes, missing IDs and no-URL behavior. Actual OS browser launch remains outside available verification; no workaround of the browser restriction.

This contract supersedes conflicting historical proposals below.

## Context

dstask's `open` command finds URLs in task notes and opens them in the browser. Tickets often reference PRs, docs, or design links — `tkt open <id>` saves the manual copy-paste.

## What to build

```bash
tkt open 03
# → Opening: https://github.com/smileynet/tkt/pull/42

tkt open 03 --list
# → URLs in ticket 03:
# →   1. https://github.com/smileynet/tkt/pull/42
# →   2. https://docs.example.com/deploy-guide
```

Behavior:
- Scans ticket body (below frontmatter) for URLs
- Opens the first URL found (or prompts if multiple with `--list`)
- Uses `xdg-open` (Linux), `open` (macOS), `start` (Windows)
- Error if no URLs found in ticket

## Acceptance criteria

- [ ] `tkt open <id>` opens first URL found in ticket body
- [ ] `tkt open <id> --list` shows all URLs numbered
- [ ] Cross-platform: xdg-open / open / start
- [ ] Error message if no URLs in ticket
- [ ] Error message if ticket not found


## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `cli`.

Deferred URL helper is absent from src/cli.rs. --list must list without launching; launch only the explicit selected/default URL and handle Windows without shell interpolation.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.


## Reactivation — 2026-11-03

The owner requested review of all parked work and execution of actionable tickets. This supersedes earlier blanket deferral. See [the parked-work review](../docs/parked-review-2026-11-03.md) for current scope and sequencing. Independent critic review is required before closure.
