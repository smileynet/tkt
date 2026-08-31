---
id: "172"
title: "Investigate: recurring skill/UX gaps (--evidence↔validation_criteria, --tags-only-at-new, TBD bodies)"
status: open
blocked_by: []
validation_criteria:
  - "Each of the 3 reported gaps assessed: is it a skill-doc gap, a binary-UX gap, or both"
  - "Decision per gap: doc fix (skill/help text), UX change (binary), or won't-fix with rationale"
  - "Any agreed doc/UX changes filed as their own follow-up tickets or applied"
---

# Investigate: recurring skill/UX gaps (--evidence↔validation_criteria, --tags-only-at-new, TBD bodies)

## Intent source

Surfaced during a `/guidance-sync` probe in the resume-buddy project (2026-08-30). Three tkt
gotchas were hit repeatedly across a ~20-ticket session by an agent following the `tkt` skill —
each cost a failed command or wasted context. They were captured in that project's AGENTS.md as a
stopgap, but they're not project-specific: the `tkt` skill (and possibly the binary's UX) should
teach/handle them so every project doesn't re-learn them.

## The three gaps (each: is it a skill-doc gap, a binary-UX gap, or both?)

1. **`--evidence` maps to `validation_criteria`, not body ACs — and the mapping is by count/index.**
   `tkt close --check-all --evidence "..."` errored twice ("criterion N does not exist") because the
   agent provided one `--evidence` per body `## Acceptance criteria` checkbox, but tkt expects one
   per `validation_criteria` frontmatter entry. Non-obvious that these are two different lists.
   - Skill angle: document the mapping explicitly (evidence ↔ validation_criteria, 1:1 by position).
   - UX angle: the error could name the expected count up front, or accept named `N=...` more forgivingly,
     or reconcile the two AC representations.

2. **`tkt edit` accepts `--priority` but NOT `--tags`.** Agent tried `tkt edit <id> --tags X` (to
   retro-tag tickets for a phase-ordering scheme) → usage error. Tags are settable only at `tkt new`.
   - Skill angle: the frontier-work steering already says "retro-tagging rarely happens / set at new,"
     but the skill could state the hard constraint that `edit` has no `--tags`.
   - UX angle: should `edit` support `--tags`/`--add-tag`/`--remove-tag`? (Deliberate omission, or gap?)

3. **`tkt new` creates a `TBD` body; agents close tickets with the stub unfilled.** validation_criteria
   set at `new` carry the real contract, so a ticket with a `TBD` "What to build / ACs" body still
   validates and still closes clean — several tickets were closed this way before a sync caught it.
   - Skill angle: the skill should stress "fill the body at `new` time" (partially in resume-buddy
     AGENTS.md now).
   - UX angle: could `tkt validate` warn on a `TBD`/empty body? (advisory, like the recent
     backlog/tag nudges in tickets 168/171.)

## Acceptance criteria

- [ ] Each of the 3 gaps assessed: skill-doc gap, binary-UX gap, or both
- [ ] Decision per gap: doc fix (skill/help text), UX change (binary), or won't-fix with rationale
- [ ] Any agreed doc/UX changes filed as their own follow-up tickets or applied

## Context / notes

- Gap 3 pairs naturally with the existing advisory-nudge pattern (tickets 168 tag-less, 171
  shared-attribute cadence) — a `TBD`-body validate warning would be consistent with that direction.
- Evidence trail: resume-buddy `.tickets/` + its AGENTS.md "Working with tkt" section.

## Out of scope

- The resume-buddy-specific AGENTS.md capture (already done there as a stopgap).

