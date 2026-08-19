---
id: "121"
title: "Deep contextual audit: evidence vs criteria, closure quality analysis"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "tkt audit --deep produces per-ticket analysis of evidence against acceptance criteria"
  - "companion skill enables agents to perform contextual ticket review"
---

# Deep contextual audit: evidence vs criteria, closure quality analysis

## Problem

The current `tkt audit` checks structural properties (unchecked ACs, TBD resolutions, stale WIP) but cannot assess QUALITY — whether evidence actually satisfies criteria, whether resolutions are meaningful, or whether agents are "gaming" close gates with low-effort compliance.

Examples of problems audit should catch:
- Evidence: "looks good" for criterion "tests pass" (no actual test output)
- All ACs checked but resolution says "TBD" or is a single word
- Evidence doesn't reference the specific criterion it claims to satisfy
- `--force` used without justification
- Closed ticket with no body changes (nothing was actually built)

## What to build

### 1. `tkt audit --deep` (CLI)

A deeper analysis mode that reads ticket CONTENT (not just frontmatter) and evaluates:

| Check | What it catches |
|-------|----------------|
| Evidence specificity | "looks good" / "done" / "works" vs actual command output |
| Evidence ↔ criteria alignment | Each evidence string should reference its criterion's domain |
| Resolution substance | Single words, "TBD", copy of title = weak |
| Force-close justification | `--force` used without a `--resolution` explaining why |
| Body unchanged | Ticket body identical to template = no work documented |
| AC checkbox ratio | If 10 ACs exist and all checked in one close = suspicious if resolution is thin |

Output: findings per ticket with severity (info/warn/error), same format as `tkt validate`.

### 2. Companion skill: `audit-quality`

A skill that agents can use for deeper, contextual review of a project's tickets:

- Read each closed ticket's body, ACs, evidence, and resolution
- Cross-reference evidence against specific criteria (semantic match, not just existence)
- Flag patterns: bulk closes, thin resolutions, repetitive evidence
- Suggest improvements for open tickets' validation criteria (are they specific enough to audit?)

This is a SKILL (agent-driven, reads files, produces recommendations) not just a CLI flag — because it benefits from the agent's ability to read and reason about prose content.

## Context

- **Relevant files:** `src/audit.rs` (existing pure audit rules), `src/commands/audit.rs` (command entry point)
- **Existing audit:** structural (ACs checked?, resolution present?, stale WIP?)
- **New audit:** semantic (evidence QUALITY, criterion SPECIFICITY, resolution SUBSTANCE)
- **Skill location:** `skills/tkt/references/audit-quality.md` or a new top-level `skills/audit-quality/SKILL.md`

## Design questions

1. Should `--deep` be a separate subcommand (`tkt audit-deep`) or a flag? Flag is simpler but conflates two different operations.
2. Should the skill invoke `tkt audit --deep` or do its own file reading? Probably both — CLI for structured findings, skill for contextual recommendations.
3. How strict should evidence matching be? Probably warn-level, not error — agents may provide valid evidence that doesn't keyword-match the criterion.

## Acceptance criteria

- [x] `tkt audit --deep` reports per-ticket quality findings
- [x] Thin evidence (< 10 chars, generic phrases) flagged
- [x] Force-close without substantial resolution flagged
- [x] Unchanged ticket body flagged (template-only)
- [x] Companion skill can invoke and interpret the audit
- [x] Exit code 1 if any findings at warn+ level (with --strict: info+ level)

## Resolution (2026-08-19)

Added --deep flag with 3 new rules (thin-evidence, force-without-justification, template-only-closure). Companion skill reference at references/audit-quality.md guides agents on semantic quality review.

### Verification
1. ✓ tkt audit --deep produces per-ticket analysis of evidence against acceptance criteria — "tkt audit --deep finds template-only-closure (8 findings) and thin-evidence rules active"
2. ✓ companion skill enables agents to perform contextual ticket review — "audit-quality.md skill reference created and deployed to all agent environments"
