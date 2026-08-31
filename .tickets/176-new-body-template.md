---
id: "176"
title: "new: clearer body template + --body flag to fill content at creation"
status: open
blocked_by: ["174"]
priority: medium
validation_criteria:
  - "tkt new writes a body with prompting section text instead of bare TBD (test: integration::new_body_template_prompts)"
  - "tkt new --body @file / --body - fills the body at creation (test: integration::new_body_from_input)"
  - "stub remains detectable by the stub-body finding when body is left unfilled (test: integration::new_body_still_detectable)"
---

# new: clearer body template + --body flag to fill content at creation

## What to build

TBD

## Acceptance criteria

- [ ] TBD
