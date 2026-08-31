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

## Context

Parent: #172. Root cause of stub tickets: `tkt new`'s body template is the stub.
`new_ticket_text` (`ticket.rs:989`) writes exactly:
`\n# {title}\n\n## What to build\n\nTBD\n\n## Acceptance criteria\n\n- [ ] TBD\n` —
only `{title}` is interpolated; `--spec`/`--validation` go to frontmatter, never the
body. Used on both create and push-retry paths, and reused by `batch`.

## Prior art (research: .scratch/research/traceability.md, dor-templates.md)

Capture content live at creation. A lightweight create-time template with prompting
section text (not bare TBD) plus an input path (`--body`) lets agents fill the real
contract at `new` time rather than leaving a stub the advisory later flags.

## What to build

1. Replace bare `TBD` with short prompting text per section (e.g. `## What to build\n\n<what & why — link the source: --spec, or a #NN/parent ref>`), keeping it detectable as unfilled.
2. Add `tkt new --body @file` / `--body -` (stdin) to fill the body at creation. `batch` inherits the template change.
3. Keep the stub detectable by #174's `stub-body` finding when left unfilled (don't defeat the advisory).

Blocked by #174 (the detector must exist so "still detectable" AC is testable).

## Acceptance criteria

- [ ] `tkt new` writes prompting section text instead of a bare `TBD` line
- [ ] `tkt new --body @file` and `--body -` fill the body at creation
- [ ] An unfilled body is still flagged by the `stub-body` finding (#174)
- [ ] `batch` uses the same template
- [ ] Guidance surfaces updated per .memory/agent-guidance-surfaces.md
- [ ] `mise run check` passes
