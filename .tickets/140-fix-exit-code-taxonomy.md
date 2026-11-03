---
id: "140"
title: "Fix exit code taxonomy leaks: config errors exit 2 instead of 1"
status: done
blocked_by: []
priority: low
validation_criteria:
  - "tkt config --set malformed exits 1; JSON output classifies it as Validation rather than Io/crash"
tags: ["config", "kind:bug", "triage:ready-for-agent", "stream:config"]
---

# Fix exit code taxonomy leaks: config errors exit 2 instead of 1

## What to build

Historical re-triage 2026-10-29: the original malformed-input case was not tested. On 0.5.0, `tkt config --set nonsense.key=1`
exits 0 and accepts the unknown key, rather than exiting 2. Accepting unknown or invalid
keys silently is the real gap, and #190 (strict typed config) covers it. Record the
original malformed input if it differs; otherwise this folds into #190.

## Acceptance criteria

- [x] `tkt config --set malformed` exits 1 with a Validation error, rather than exit 2/crash.
- [x] `-o json` reports the same structured Validation error on stderr.
- [x] Valid key=value writes retain their existing behavior; invalid known values remain #190's scope.

## Triage — 2026-11-02

Disposition: `kind:bug`, `triage:ready-for-agent`; stream `config`.

Confirmed independently of #190: config --set malformed exits 2 with crash: expected key=value format. Restore input-validation exit 1 and structured Validation error; reopen as actionable work.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Resolution (2026-11-03)

Malformed assignment now returns structured Validation exit 1; valid writes preserved. Independent critic accepted; docs/evidence/lifecycle-identifiers-config-241-164-140.md.

### Verification
1. ✓ tkt config --set malformed exits 1; JSON output classifies it as Validation rather than Io/crash — "Two config_errors integration tests pass, including JSON and dry-run no-write; full gate passes322 Rust/42Node with one existing skip."
