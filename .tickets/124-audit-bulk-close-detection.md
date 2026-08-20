---
id: "124"
title: "Detect bulk-close sessions in audit skill (same-session generic evidence)"
status: backlog
blocked_by: []
priority: medium
validation_criteria:
  - "audit skill detects and flags tickets closed in same session with identical/generic evidence"
  - "guidance distinguishes legitimate batch work from gaming"
---

# Detect bulk-close sessions in audit skill (same-session generic evidence)

## Problem

crew-research #117 and #118 were bulk-closed in the same session with identical generic evidence ("dry-run passes", "mise run validate: 0 errors"). Each claimed specific work (7 eval conversions, 5 boundary fixes) but the evidence only proved the system didn't break — not that the specific changes were made.

## What to build

Add guidance to the audit-quality skill for detecting bulk-close patterns:

**Signals:**
- Multiple tickets closed within minutes of each other
- Evidence strings that are identical or near-identical across tickets
- Generic validation evidence ("tests pass", "validate: 0 errors") used for specific functional claims
- Resolution text that's identical across tickets ("Done")

**Guidance for the reviewing agent:**
- Same-session closes are fine IF evidence is ticket-specific
- Generic system-health evidence is fine for infra/config tickets, suspicious for feature tickets
- "Done" as resolution is always a flag for contextual review

This is SKILL-ONLY (not CLI) because distinguishing legitimate batch work from gaming requires reading the ticket titles, understanding what specific evidence would be appropriate, and assessing whether generic evidence is sufficient for the claimed scope.

## Context

- **Evidence:** crew-research #117/#118 contextual review in `.scratch/audit-results-batch1.md`
- **Relevant file:** `skills/tkt/references/audit-quality.md`

## Acceptance criteria

- [ ] Audit skill guidance includes bulk-close detection criteria
- [ ] Guidance distinguishes legitimate batch work from gaming
- [ ] Provides specific examples (what to flag, what to accept)
