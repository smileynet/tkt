---
id: "126"
title: "Confirm and address Codex review findings through 2c0c021"
status: in_progress
blocked_by: []
priority: high
validation_criteria:
  - "Every finding is independently confirmed, rejected, or obsolete"
  - "Confirmed findings are corrected with regression coverage"
  - "Relevant build, test, and lint checks pass"
---

# Confirm and address Codex review findings through 2c0c021

## Review provenance

- Reporter: Codex
- Review run: `e945f41f-a977-408c-8bfc-3a356553e525`
- Review target: `2c0c02142c577b3ec09e840b9ad257a3ca62e2f1`
- Review coverage: `7082cd1a5e9ce89514a3c80645af01a1292354e4..2c0c02142c577b3ec09e840b9ad257a3ca62e2f1`
- Confirmation status: unconfirmed

These findings were produced by Codex. They are reviewer hypotheses, not
established defects. The agent working this ticket must reproduce and confirm
each finding against current code before changing it.

## Findings

### F1 — high: legacy default environment can hide tickets whose configured capabilities satisfy `requires`

- Location: `src/commands/ready.rs:12`
- Evidence: `ready` first calls `frontier_with_default_env`, which filters every non-empty `requires` list against the single legacy `ready.default_env` value, and only afterward filters the survivors against `machine.capabilities`. With `ready.default_env=corp`, `machine.capabilities=gpu,linux`, and `requires:[gpu]`, the first filter removes the ticket even though the documented capability subset is satisfied.
- Risk: valid frontier work silently disappears on machines that retain a legacy default environment while adopting the new capability configuration.
- Suggested confirmation: Add a focused frontier/CLI test combining a non-empty `ready.default_env` with different matching `machine.capabilities` and assert that a ticket requiring the configured capability remains visible.
- Codex confidence: verified

### F2 — medium: persisted context state is not ignored by Git

- Location: `src/context.rs:109`
- Evidence: `context::save` writes `.tickets/.context`, while the repository `.gitignore` has no rule for that path. In a clean initialized project, `tkt context +backend` therefore creates an untracked file despite the module contract describing this as repo-local session state.
- Risk: ordinary context changes dirty worktrees, can be accidentally committed, and can leak one contributor's personal filter to other contributors.
- Suggested confirmation: In a temporary initialized repository, run `tkt context +backend` and assert both that the context works and that `git status --porcelain` remains empty.
- Codex confidence: verified

### F3 — high: migration applies destructive conversion without the required first dry run

- Location: `src/commands/migrate.rs:126`
- Evidence: ticket 77's safety contract says `--dry-run` is required on the first run, but the command applies immediately whenever the global dry-run flag is absent. The apply path removes each source before writing its target and also warns that unresolved dependencies will be dropped; there is no confirmation or persisted proof that the plan was previewed.
- Risk: a mistyped or prematurely run migration can rename and rewrite an entire ticket corpus and silently discard dependency edges before the operator has reviewed the mapping.
- Suggested confirmation: Create a fresh tk-format corpus, invoke `tkt migrate --from tk` without a prior dry run, and verify whether the command refuses before modifying files; include an unresolved dependency in the fixture.
- Codex confidence: verified

### F4 — medium: telemetry omits several explicitly required notable flags

- Location: `src/cli.rs:586`
- Evidence: ticket 116 names `requires`, `all`, and `dry-run` among flags to track. `notable_flags` ignores `requires` in both `new` and `batch`, treats `Telemetry` as having no notable optional flags (so `--all` is omitted), and never incorporates the global `--dry-run` value.
- Risk: telemetry cannot measure adoption of the new machine-requirements workflow, full telemetry inspection, or safety previews, undermining the stated purpose of the feature.
- Suggested confirmation: With telemetry enabled in an isolated config directory, run commands using each flag and inspect the resulting event's `flags` array.
- Codex confidence: verified

## Acceptance criteria

- [ ] Every finding is independently marked confirmed, rejected, or obsolete
- [ ] Rejected or obsolete findings include evidence and rationale
- [ ] Confirmed findings are corrected
- [ ] Regression tests cover confirmed defects where practical
- [ ] Relevant build, test, and lint checks pass
- [ ] Corrected changes receive a fresh review
