---
id: "92"
title: "Implement validation_criteria field + evidence-gated close"
status: open
blocked_by: ["91"]
priority: high
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

- [ ] `validation_criteria` parsed from frontmatter as Vec<String>
- [ ] `tkt new --vc "..."` creates tickets with validation_criteria
- [ ] `tkt edit --vc "..."` replaces the criteria list
- [ ] `tkt close --evidence "..."` maps evidence to criteria (positional)
- [ ] Named evidence `N=...` maps to specific criterion
- [ ] Config: `require_validation_criteria` (default false)
- [ ] Config: `require_validation_evidence` (default "warn")
- [ ] `--force` bypasses evidence gate
- [ ] Resolution section shows criterion + evidence pairs
- [ ] `tkt audit` reports low-evidence closures
- [ ] All existing tests still pass (backward compatible)
- [ ] Our .tickets/config.toml requires both

# Implement validation_criteria field + evidence-gated close

## What to build

TBD

## Acceptance criteria

- [ ] TBD
