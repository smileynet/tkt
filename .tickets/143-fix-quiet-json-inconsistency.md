---
id: "143"
title: "Fix -q + -o json emits bare non-JSON output inconsistently"
status: done
blocked_by: []
priority: low
validation_criteria:
  - "all mutations produce a structured JSON success object under -o json -q"
tags: ["cli", "kind:bug", "triage:ready-for-agent", "stream:quality"]
assignee: "codex"
---

# Fix -q + -o json emits bare non-JSON output inconsistently

## What to build

Observed on 0.5.0 (`a827bb6`, 2026-10-29): `tkt new d --title Delta -q -o json` printed a
bare `04`, not JSON. Scripts that ask for `-o json` must always get JSON; `-q` should
shrink the object (for example `{"id":"04"}`), not change the format.

## Acceptance criteria

- [x] Every mutating command run with `-o json -q` prints exactly one structured JSON success object (never a bare ID or number)
- [x] A test iterates the mutating commands under `-o json -q`
- [x] Help documents how `-q` and `-o json` combine

## Triage — 2026-11-02

Disposition: `kind:bug`, `triage:ready-for-agent`; stream `cli`.

Confirmed: new quiet-json --title Quiet JSON -q -o json prints bare 02. Require the structured success envelope. IDs without leading zero can parse as JSON numbers, so syntax-only testing is insufficient.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Resolution (2026-11-03)

Write-capable commands retain structured quiet-JSON outcomes, including no-ops and previews; help documents precedence. Independent critic accepted. Evidence promoted; shared build cache retained for maintenance, delivery helpers expire after dashboard publication.

### Verification
1. ✓ all mutations produce a structured JSON success object under -o json -q — "Quiet JSON matrix covers 19 writer cases and previews/no-ops; final gate 384 Rust and 42 Node passes with one existing Windows skip. Installed preview returns one JSON object. See docs/evidence/quiet-json-context-143-144.md."
