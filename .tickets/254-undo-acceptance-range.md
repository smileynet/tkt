---
id: "254"
title: "Restrict undo close checkbox recognition to the acceptance section"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "The public red regression is refused without effects after restricting recognition to the actual acceptance range"
  - "Supported close transitions and existing undo exact-inverse and failure controls remain compatible"
  - "Fresh full gate and independent critic review accept final source"
tags: ["stream:quality", "kind:bug", "triage:ready-for-agent"]
parent: "191"
assignee: "codex"
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

- [x] A forged close changing a checkbox outside the actual acceptance section is refused without effects.
- [x] Legitimate close checkbox changes and all existing undo recovery controls still pass.
- [x] Final local gate and independent critic accept the repair before closure.

## Resolution (2026-11-04)

Restricted close undo recognition to the actual acceptance section; the direct forged Background checkbox regression now refuses without effects. Final gate and critic accepted.

### Verification
1. ✓ The public red regression is refused without effects after restricting recognition to the actual acceptance range — "The public regression returned success before the fix and now returns refusal. It updates receipt identities so it tests the actual semantic boundary, not stale metadata. See docs/evidence/undo-range-254.md."
2. ✓ Supported close transitions and existing undo exact-inverse and failure controls remain compatible — "All nine undo public tests and the full existing suite pass, preserving legitimate checkbox changes, exact inverse and hook/push failure handling. Recognition reuses core::ac_section_range from the close writer."
3. ✓ Fresh full gate and independent critic review accept final source — "Fresh mise run check: 444 Rust tests and 42 Node fixtures passed, fmt clean and clippy zero warnings; one existing Windows skip. Independent critic accepted final source. Source hashes promoted and disposable task files recycled; shared build output retained for ongoing work."
