---
id: "177"
title: "guidance: state evidence↔validation_criteria rule, tag editing, fill-body-at-new, link-the-source"
status: done
blocked_by: ["176", "233"]
priority: high
validation_criteria:
  - "a surface states the evidence↔validation_criteria 1:1-by-index rule plainly (test: manual doc review)"
  - "init.rs snippets instruct fill-body-at-new and link-the-source (test: manual doc review)"
  - "tag-edit guidance matches the replace/add/remove/clear interface delivered by #233"
tags: ["guidance", "kind:enhancement", "triage:ready-for-agent", "stream:guidance"]
assignee: "codex"
---

# guidance: state evidence↔validation_criteria rule, tag editing, fill-body-at-new, link-the-source

## Context

Parent: #172 (doc halves of gaps 1, 2, 3). Review (`.scratch/review/guidance-body.md`)
found the init.rs snippets — the highest-traffic surface, injected into every project —
are silent on all three, and no surface states the evidence↔validation_criteria rule
plainly (the direct cause of the "criterion N does not exist" confusion).

## What to build

Propagate across all guidance surfaces per `docs/agents/agent-guidance-surfaces.md`
(init.rs snippets, SKILL.md, commands.md, ticket-format.md, ticket-standards.md,
AGENTS.md, README):

1. **evidence↔validation_criteria** — state plainly: body `## Acceptance criteria`
   checkboxes ← `--check-all`/`--ac`; frontmatter `validation_criteria` ← `--evidence`
   1:1 by index; the counts are independent.
2. **fill the body at `tkt new`** — don't ship the `TBD` stub (pairs with #174/#176).
3. **link the source** — `--spec`, or a `#NN`/parent reference in the body (pairs with #175).
4. **Tag editing** — describe the replace/add/remove/clear contract delivered by #233; do not call the existing gap intentional.

Best sequenced after #174/#176 land so the docs describe the new behavior.

## Disposition update (2026-10-10)

#179 completed the first criterion's evidence-index guidance. It is recorded checked here;
the remaining documentation pass is intentionally blocked by #176 so it can describe the
finished new-ticket body flow rather than an interim interface.

## Acceptance criteria

- [x] A surface states the evidence↔validation_criteria 1:1-by-index rule plainly
- [x] init.rs snippets instruct fill-body-at-new and link-the-source
- [x] Tag-edit guidance matches #233 across the required surfaces.
- [x] All surfaces synced per the guidance-surfaces checklist; `deploy-skills.sh` run
- [x] Coordinate any version bump through the release protocol; never hand-edit version fields.

## Added scope (2026-10-29, grey-matter ticket 101)

From the agent-harness-research workflow guidance (recommendations 2 and 12): the close
guidance should ask evidence to say what was checked, what outcome each check produced
(pass, failure, skip, infrastructure error) and what remains unknown, rather than only
"tests pass". This refinement is owned by #230; do not implement it twice here.

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `guidance`.

Retain dependency #176; remove the claim that absent edit --tags is intentional. #233 owns tag-edit behavior; #230 owns classified evidence guidance. #179 already delivered index mapping.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Resolution (2026-11-03)

All instruction/reference surfaces now map body ACs and indexed frontmatter evidence independently, fill body/source at creation and document tag edits. Reconcile/orient lifecycle guidance corrected. Critic accepted. Evidence promoted; fixtures self-clean/shared target retained. docs/evidence/hooks-cancel-guidance-235-227-177.md.

### Verification
1. ✓ a surface states the evidence↔validation_criteria 1:1-by-index rule plainly (test: manual doc review) — "Core/work/init/commands/format/standards/README/AGENTS explicitly state independent AC versus validation/evidence counts."
2. ✓ init.rs snippets instruct fill-body-at-new and link-the-source (test: manual doc review) — "All six init targets direct creation body input and source citation; no version fields changed."
3. ✓ tag-edit guidance matches the replace/add/remove/clear interface delivered by #233 — "Replace/clear/add/remove ordering matches233; full gate375Rust and applicable post-doc Node31pass1skip, native validation and deployment passed."
