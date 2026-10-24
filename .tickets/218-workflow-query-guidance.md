---
id: "218"
title: "Use supported tag context in workflow query examples"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "Distributed workflow examples no longer invoke unsupported query --tag; scoped query examples use the actual context contract."
  - "Isolated candidate CLI commands prove tag context filters queries and restoring context preserves the prior scope."
  - "Fresh guidance/prose checks and applicable tooling tests pass, with a regenerated candidate tracked by grey-matter 69."
tags: ["stream:tooling"]
---

# Use supported tag context in workflow query examples

## Intent source

Found while executing grey-matter release-readiness ticket 69 on 2026-10-24.
The built 0.5.0 candidate cd96042 rejects `tkt query --tag ink` with exit 2 and
"unexpected argument --tag". Its capabilities omit query flags and the help
confirms no --tag; `src/commands/query.rs` instead applies the active context and
does not serialize tags. The invalid examples occur in tkt-reconcile twice and
tkt-work once. This is a guidance fix, not a new query flag.

## What to build

Make workflow examples executable against the current CLI: inspect tag vocabulary
from ticket frontmatter, scope queries with the existing context command, and
restore any previous context after temporary scoping. Preserve the documented
inclusion of untagged tickets. Do not mutate query behavior or add a --tag alias.

Read `skills/tkt-reconcile/SKILL.md`, `skills/tkt-work/SKILL.md`,
`src/commands/query.rs`, and `src/context.rs`. The migration reference's --map
example is explicitly labeled future work, and the ready-to-close command arrows
were extraction false positives; neither is a confirmed removed CLI flag.
Coordinated candidate regeneration/evidence belongs to grey-matter 69.

## Acceptance criteria

- [ ] No distributed workflow invokes query --tag; examples and vocabulary instructions match the actual CLI.
- [ ] Isolated CLI replay proves context filtering, untagged-ticket inclusion and restoration of the prior scope.
- [ ] Prose and applicable tooling checks pass; release verification records the replacement candidate requirement.
