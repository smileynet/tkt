---
id: "174"
title: "validate: advisory stub-body finding on open/in_progress tickets (promote check_template_only)"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "validate emits advisory stub-body warning for open ticket with TBD template body, exit 0 (test: integration::validate_stub_body_advisory)"
  - "validate --strict escalates stub-body to failure exit 1 (test: integration::validate_stub_body_strict)"
  - "non-stub open ticket produces no stub-body finding (test: integration::validate_no_false_stub)"
---

# validate: advisory stub-body finding on open/in_progress tickets (promote check_template_only)

## What to build

TBD

## Acceptance criteria

- [ ] TBD
