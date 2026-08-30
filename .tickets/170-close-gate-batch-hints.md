---
id: "170"
title: "close: batch unmet gates into one message + populate hints + fix G5 mis-kinding"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "close with multiple unmet gates lists all in one message (test: integration::close_gates_batched)"
  - "each close gate populates a hint naming the missing flag (test: integration::close_gate_hints)"
  - "G5 partial-evidence emits err=gate_failed not validation (test: integration::close_partial_evidence_kind)"
---

# close: batch unmet gates into one message + populate hints + fix G5 mis-kinding

## What to build

TBD

## Acceptance criteria

- [ ] TBD
