---
id: "108"
title: "Confirm and address Codex review findings through 7082cd1"
status: done
blocked_by: []
priority: high
validation_criteria: 
  - "F1 confirmed and fixed"
  - "regression test passes"
  - "cargo test 56 passed"
---

# Confirm and address Codex review findings through 7082cd1

## Review provenance

- Reporter: Codex
- Review run: `beeac31e-28f0-4a48-b114-8966a4ee3734`
- Review target: `7082cd1a5e9ce89514a3c80645af01a1292354e4`
- Review coverage: `260c09e7ac9feffa959a9754b981dcce52c9e004..7082cd1a5e9ce89514a3c80645af01a1292354e4`
- Confirmation status: unconfirmed

These findings were produced by Codex. They are reviewer hypotheses, not
established defects. The agent working this ticket must reproduce and confirm
each finding against current code before changing it.

## Findings

### F1 — medium: duplicate named evidence bypasses the complete-evidence close gate

- Location: `src/commands/close.rs:72`
- Evidence: With two `validation_criteria` and `close.require_validation_evidence = "true"`, closing with `--evidence 1=one --evidence 1=overwrite` exits 0 and records the second criterion with empty evidence. `parse_evidence` overwrites the first slot, while the gate checks only `evidence.len()` rather than populated criterion slots.
- Risk: A ticket can be marked done despite the configured requirement that every validation criterion have evidence, weakening closure auditability and allowing accidental duplicate indices to conceal missing verification.
- Suggested confirmation: Add an integration test that closes a two-criterion ticket with the same named evidence index twice and assert a domain failure identifying the missing/duplicate criterion.
- Codex confidence: verified

## Acceptance criteria

- [x] Every finding is independently marked confirmed, rejected, or obsolete
- [x] Rejected or obsolete findings include evidence and rationale
- [x] Confirmed findings are corrected
- [x] Regression tests cover confirmed defects where practical
- [x] Relevant build, test, and lint checks pass
- [x] Corrected changes receive a fresh review

## Resolution (2026-08-18)

F1 confirmed and fixed. parse_evidence now rejects duplicate named indices and verifies all slots are filled. Regression test added.

### Verification
1. ✓ F1 confirmed and fixed — "F1 confirmed: reproduced by test — duplicate 1=foo 1=bar bypassed gate. Fixed: now rejects with 'duplicate evidence' error"
2. ✓ regression test passes — "test_evidence_duplicate_named_index_rejected: 1 passed (regression test)"
3. ✓ cargo test 56 passed — "cargo test: 56 passed, 0 failed, cargo clippy: 0 warnings"
