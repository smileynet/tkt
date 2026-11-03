---
id: "151"
title: "load_corpus ejection is stderr-only, bypasses JSON envelope and TKT_ASCII/sym_warn"
status: open
blocked_by: []
priority: low
validation_criteria:
  - "ejected-ticket warnings honor -o json (structured), TKT_ASCII, and sym_warn"
  - "consumers can distinguish empty corpus from corpus-with-ejected-tickets"
tags: ["parser", "kind:bug", "triage:ready-for-agent", "stream:quality"]
---

# load_corpus ejection is stderr-only, bypasses JSON envelope and TKT_ASCII/sym_warn

## What to build

When `load_corpus` skips an unparseable ticket it emits a hard-coded `eprintln!("⚠ skipping ...")` — bypassing `sym_warn`/`TKT_ASCII` (so the ✓/⚠ symbols ignore ASCII mode) and the `-o json` structured-error envelope. Every consumer except standalone `validate` treats an ejected ticket as nonexistent, and a dependent ticket's blocker silently vanishes from the frontier with no machine-readable signal.

Make ejection diagnostics first-class: route through `sym_warn`, honor `TKT_ASCII`, and surface in the JSON envelope when `-o json`. Consider returning skip diagnostics from `load_corpus` (rather than eprintln) so callers can decide how to present them.

## Context

- **Relevant files:** `src/core/ticket.rs` (load_corpus:~548), `src/color.rs` (sym_warn), `src/cli.rs` (JSON envelope)
- **Discovered during #132** — orthogonal to the parser leniency fix
- **Coupling with #150:** if load_corpus returns diagnostics, doctor (#150) could consume them directly instead of a separate loop

## Acceptance criteria

- [ ] ejected-ticket warnings honor -o json (structured), TKT_ASCII, and sym_warn
- [ ] consumers can distinguish empty corpus from corpus-with-ejected-tickets
- [ ] existing corpus-loading behavior unchanged for well-formed tickets

## Out of scope

- doctor detection logic (#150)

## Triage — 2026-11-02

Disposition: `kind:bug`, `triage:ready-for-agent`; stream `parser`.

Confirmed: TKT_ASCII=1 tkt -o json query on a ticket lacking id exits 0 with empty stdout and a Unicode warning on stderr. Preserve valid-corpus behavior while exposing skipped-ticket diagnostics.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.
