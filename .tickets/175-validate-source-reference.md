---
id: "175"
title: "validate: advisory no-source-reference finding (union of spec/#NN/parent/link)"
status: open
blocked_by: ["174"]
priority: medium
validation_criteria:
  - "validate warns when a ticket has no spec, no blocked_by, and no #NN/docs/.memory reference in body (test: integration::validate_no_source_ref)"
  - "any one reference kind satisfies the check (test: integration::validate_source_ref_union)"
  - "advisory by default (exit 0); [validate] config key opts into hard gate (test: integration::validate_source_ref_gate)"
---

# validate: advisory no-source-reference finding (union of spec/#NN/parent/link)

## What to build

TBD

## Acceptance criteria

- [ ] TBD
