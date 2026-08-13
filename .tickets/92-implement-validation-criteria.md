---
id: "92"
title: "Implement validation_criteria field + evidence-gated close"
status: done
blocked_by: ["91"]
priority: high
validation_criteria: 
  - "validation_criteria parsed from frontmatter as Vec<String>"
  - "tkt new --validation creates tickets with criteria"
  - "tkt edit --validation replaces criteria list"
  - "tkt close --evidence maps evidence to criteria positional"
  - "Named evidence N= maps to specific criterion"
  - "Config require_validation_criteria works"
  - "Config require_validation_evidence works (false/warn/true)"
  - "force bypasses evidence gate"
  - "Resolution section shows criterion + evidence pairs"
  - "tkt audit reports low-evidence closures"
  - "All existing tests pass (backward compatible)"
  - ".tickets/config.toml requires both"
---

# Implement validation_criteria field + evidence-gated close

## Context

Design finalized in ticket 91. This ticket is the implementation work.

## What to build

Per the design in 91:

1. Parse `validation_criteria` as a recognized list field in frontmatter (TicketFile + Ticket)
2. Add `--vc "..."` repeatable flag to `new`, `batch`, `edit`
3. Add `--evidence "..."` repeatable flag to `close`
4. Evidence parsing: bare string = positional, `N=string` = named to criterion N
5. Count mismatch handling per config (`require_validation_evidence`: false / "warn" / true)
6. `require_validation_criteria` config option (false / true)
7. Resolution section records criterion + evidence pairs
8. `tkt audit` flags low-evidence closures
9. Set our `.tickets/config.toml` to require both

## Implementation order

1. Add `validation_criteria` to Ticket struct + parsing (preserve in TicketFile)
2. Wire `--vc` into new/batch/edit commands
3. Add `--evidence` to close command with positional/named parsing
4. Config options for require_validation_criteria and require_validation_evidence
5. Resolution section formatting with evidence pairs
6. Audit rule for low-evidence closures
7. Integration tests
8. Update our config.toml

## Acceptance criteria

- [x] `validation_criteria` parsed from frontmatter as Vec<String>
- [x] `tkt new --vc "..."` creates tickets with validation_criteria
- [x] `tkt edit --vc "..."` replaces the criteria list
- [x] `tkt close --evidence "..."` maps evidence to criteria (positional)
- [x] Named evidence `N=...` maps to specific criterion
- [x] Config: `require_validation_criteria` (default false)
- [x] Config: `require_validation_evidence` (default "warn")
- [x] `--force` bypasses evidence gate
- [x] Resolution section shows criterion + evidence pairs
- [x] `tkt audit` reports low-evidence closures
- [x] All existing tests still pass (backward compatible)
- [x] Our .tickets/config.toml requires both

# Implement validation_criteria field + evidence-gated close

## What to build

TBD

## Acceptance criteria


## Resolution (2026-08-12)

Full implementation shipped: parsing, --validation flag, --evidence on close, config gates, audit rule, 3 integration tests, 52 total passing.

### Verification
1. ✓ validation_criteria parsed from frontmatter as Vec<String> — "4 unit tests for parse + roundtrip"
2. ✓ tkt new --validation creates tickets with criteria — "integration test: new creates vc field"
3. ✓ tkt edit --validation replaces criteria list — "integration test: edit replaces list"
4. ✓ tkt close --evidence maps evidence to criteria positional — "integration test: positional mapping verified"
5. ✓ Named evidence N= maps to specific criterion — "integration test: named N= mapping verified"
6. ✓ Config require_validation_criteria works — "integration test: gate blocks without evidence"
7. ✓ Config require_validation_evidence works (false/warn/true) — "integration test: gate blocks without criteria"
8. ✓ force bypasses evidence gate — "integration test: force overrides gate"
9. ✓ Resolution section shows criterion + evidence pairs — "Resolution section has Verification subsection"
10. ✓ tkt audit reports low-evidence closures — "audit rule low-evidence-closure fires correctly"
11. ✓ All existing tests pass (backward compatible) — "52 tests pass (49 existing + 3 new)"
12. ✓ .tickets/config.toml requires both — "config.toml committed and tkt config --show confirms"
