---
id: "297"
title: "Add post-pass diff simplification step to tkt-work before ticket close"
status: done
blocked_by: []
validation_criteria:
  - "skills/tkt-work/SKILL.md includes a bounded post-pass simplification step before closing while keeping description <= 500 chars"
  - "node tools/check-tools.mjs and tkt validate --strict pass, and node tools/deploy.mjs deploys cleanly"
tags: ["stream:tooling", "kind:enhancement"]
---
## Intent source
Cross-repo adoption of prior art from https://github.com/SprocketLab/slop-code-bench (`plan-and-test.jinja` step 6 / `plan_first.jinja` step 4: simplify after tests pass).

## What to build
Add a bounded **Simplify before close** sub-step to `skills/tkt-work/SKILL.md` (under `## Working a Ticket`, after initial tests pass and before closing) instructing the agent to scan its ticket diff for newly introduced trivial wrappers, single-use clutter, narration comments, silent fallbacks, or complexity piled onto an already-large file, simplify locally without widening scope, and re-run affected checks so closure evidence covers the final bytes. Keep YAML frontmatter `description` unchanged (`<= 500` chars).

## Acceptance criteria
- [x] `skills/tkt-work/SKILL.md` documents the post-pass diff simplification step before `tkt close` while preserving frontmatter length and existing close gates
- [x] `node tools/check-tools.mjs` and `tkt validate --strict` pass and `node tools/deploy.mjs` deploys cleanly

## Resolution (2026-11-25)

Added post-pass diff simplification step before closing in skills/tkt-work/SKILL.md.

### Verification
1. ✓ skills/tkt-work/SKILL.md includes a bounded post-pass simplification step before closing while keeping description <= 500 chars — "skills/tkt-work/SKILL.md step 7 adds bounded post-pass diff simplification before verification/close; YAML frontmatter description unchanged (<= 500 chars)"
2. ✓ node tools/check-tools.mjs and tkt validate --strict pass, and node tools/deploy.mjs deploys cleanly — "node tools/check-tools.mjs (58/58 pass) and tkt validate --brief passed; node tools/deploy.mjs executed"
