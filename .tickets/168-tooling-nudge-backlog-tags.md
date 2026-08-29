---
id: "168"
title: "Optional tooling nudge: advisory on tag-less/backlog tickets"
status: backlog
blocked_by: ["167"]
priority: low
validation_criteria:
  - "tkt new prints a non-blocking advisory to stderr when a ticket is created with no tags (gated on !quiet)"
  - "validate or doctor emits an advisory (not error) counting untagged live tickets, via a check_missing_tags finding in findings.rs"
  - "decision recorded on whether to add a new.default_status config key (mirroring new.default_priority) or keep the hard-coded open default"
---

# Optional tooling nudge: advisory on tag-less/backlog tickets

## What to build

TBD

## Acceptance criteria

- [ ] TBD
