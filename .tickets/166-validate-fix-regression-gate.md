---
id: "166"
title: "validate --fix: abort if a fix introduces new findings (non-regressing gate)"
status: open
blocked_by: ["140"]
priority: high
validation_criteria:
  - "validate --fix compares finding identities (file+rule) before/after; aborts exit 1 if any NEW finding appears (test: fix::regression_gate_trips)"
  - "normal fixes that only reduce findings are unaffected (existing --fix tests stay green)"
  - "regression abort advises git checkout .tickets/ and does not use a new exit code (stays within 0/1/2 taxonomy)"
tags: ["contract"]
---

# validate --fix: abort if a fix introduces new findings (non-regressing gate)

## What to build

TBD

## Acceptance criteria

- [ ] TBD
