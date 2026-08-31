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

## Context

Parent: #172 gap 3. `tkt new` writes a stub body by default (`ticket.rs:989`):
`## What to build\n\nTBD` + `- [ ] TBD`. Every body-aware health check fires only
on `status==Done`, and the only stub detector — `check_template_only`
(`audit.rs:211`) — is opt-in (`--deep`), warning-level, and post-hoc (iterates done
tickets, inspects `## Resolution`). So a ticket is born a stub and can stay one
through the whole lifecycle. Corpus: 9 open tickets currently carry a genuine TBD
stub body (79, 83, 86, 87, 88, 89, 90, 93, 138); 12 done tickets closed clean with one.

## Prior art (research: .scratch/research/stub-detection.md)

Strongest low-FP signal = required-section-present + section-empty/placeholder.
Anchoring on the literal template sentinel (`## What to build\n\nTBD`) is a token
humans don't write → ~0% false positives (GitHub's "TODOCS" pattern). Advisory vs
blocking per Danger.js: same detection, severity chosen per rule.

## What to build

Add a `stub-body` finding to `validate` that fires on open/in_progress tickets (not
just done). Reuse the existing sentinel match from `check_template_only`
(`audit.rs`). Advisory (warning, exit 0) by default; escalate to failure under
`--strict` (matches the existing validate severity model at findings.rs
`status_from_findings`). This surfaces the 9 open stubs immediately via the command
the guidance already promotes (`tkt validate --brief`).

Follow the advisory-first pattern established by sync-plan (#169) and the batch nudge (#171).

## Implementation notes

- Detection: `body.contains("## What to build\n\nTBD")` OR `body.contains("- [ ] TBD")`
  (same as audit.rs:222-224), applied to open/in_progress tickets.
- Keep audit's done-only `template-only-closure` finding as-is (post-close signal);
  this is the pre-close counterpart.
- Check overlap with #154 (hand-flipped-done finding) — both add validate findings.

## Acceptance criteria

- [ ] `validate` emits an advisory `stub-body` warning for an open ticket with a TBD template body; exit 0
- [ ] `validate --strict` escalates `stub-body` to a failure (exit 1)
- [ ] A non-stub open ticket produces no `stub-body` finding (no false positive)
- [ ] Existing audit `template-only-closure` (done-only) behavior unchanged
- [ ] `mise run check` passes (fmt --check, clippy -D warnings, tests)
