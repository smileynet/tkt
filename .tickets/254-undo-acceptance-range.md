---
id: "254"
title: "Restrict undo close checkbox recognition to the acceptance section"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "The public red regression is refused without effects after restricting recognition to the actual acceptance range"
  - "Supported close transitions and existing undo exact-inverse and failure controls remain compatible"
  - "Fresh full gate and independent critic review accept final source"
tags: ["stream:quality", "kind:bug", "triage:ready-for-agent"]
parent: "191"
---
# Match undo recognition to the actual acceptance section

## Intent source

Follow-up to completed #79, discovered while preparing #83. The existing
close/check-all path changes only core::ac_section_range, while undo's structural
close recognizer currently allows unchecked-to-checked substitutions anywhere in
the preserved original body.

## Confirmation

The public regression forged_close_checkbox_change_outside_acceptance_is_refused
creates a genuine close, changes an unchecked Background example, and updates the
receipt's tree/blob identities. On the current implementation, --dry-run undo
returns 0 instead of the required refusal. The original specification's unrelated
checkbox is therefore accepted as a generated close mutation shape. No real
project history was reverted; the failing control uses a private fixture.

## What to build

Use the same acceptance-section range as the close writer when recognizing allowed
checkbox changes. Preserve valid new/claim/close recognition, exact inverse
publication and all history/push limits. This is structural recognition hardening,
not cryptographic author authentication or a new operation family.

## Acceptance criteria

- [ ] A forged close changing a checkbox outside the actual acceptance section is refused without effects.
- [ ] Legitimate close checkbox changes and all existing undo recovery controls still pass.
- [ ] Final local gate and independent critic accept the repair before closure.
