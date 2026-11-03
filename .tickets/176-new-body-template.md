---
id: "176"
title: "new: clearer body template + --body flag to fill content at creation"
status: done
blocked_by: ["174"]
priority: medium
validation_criteria:
  - "tkt new writes a body with prompting section text instead of bare TBD (test: integration::new_body_template_prompts)"
  - "tkt new --body @file / --body - fills the body at creation (test: integration::new_body_from_input)"
  - "new and batch support named body templates from .tickets/templates with clear missing-template errors"
  - "stub remains detectable by the stub-body finding when body is left unfilled (test: integration::new_body_still_detectable)"
tags: ["cli", "kind:enhancement", "triage:ready-for-agent", "stream:workflow"]
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

- [x] `tkt new` writes prompting section text instead of a bare `TBD` line
- [x] `tkt new --body @file` and `--body -` fill the body at creation
- [x] An unfilled body is still flagged by the `stub-body` finding (#174)
- [x] `new --template <name>` and `batch --template <name>` read `.tickets/templates/<name>.md`; missing templates fail clearly and default behavior works without that directory.
- [x] `batch` uses the same default template
- [x] Guidance surfaces updated per docs/agents/agent-guidance-surfaces.md
- [x] `mise run check` passes

## Absorbs #87 (2026-10-29)

`tkt new --template` (#87) is merged into this ticket: a clearer default body and named
templates are one feature. #87 waits on #227 to cancel after ADR 0004.

## Field report — 2026-11-01

Confirmed in philippines-trip: every `tkt new` left a "TBD" body that then needed a hand edit. A `--body`/`--body-file` flag would remove that step.

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `cli`.

new --help has no body/template flags and creation still emits TBD. #87's named-template scope must be represented explicitly in ACs; keep #174 stub detection working.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Resolution (2026-11-03)

Creation accepts full body literal/file/stdin and named templates for new/batch, with captured retry content and detectable prompting defaults. Critic accepted source/guidance. Evidence promoted docs/evidence/project-owner-body-165-194-176.md; fixtures self-clean shared caches retained.

### Verification
1. ✓ tkt new writes a body with prompting section text instead of bare TBD (test: integration::new_body_template_prompts) — "Public body_input default prompt test red before and pass after."
2. ✓ tkt new --body @file / --body - fills the body at creation (test: integration::new_body_from_input) — "File/stdin and literal creation paths pass including paths with spaces and dry-run no writes."
3. ✓ new and batch support named body templates from .tickets/templates with clear missing-template errors — "New/batch named templates use validated names; missing/unsafe names fail before writes."
4. ✓ stub remains detectable by the stub-body finding when body is left unfilled (test: integration::new_body_still_detectable) — "New and legacy prompt detection works; full gate361Rust42Nodepass one existing skip; native Claude checks passed."
