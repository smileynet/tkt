---
id: "218"
title: "Use supported tag context in workflow query examples"
status: done
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

- [x] No distributed workflow invokes query --tag; examples and vocabulary instructions match the actual CLI.
- [x] Isolated CLI replay proves context filtering, untagged-ticket inclusion and restoration of the prior scope.
- [x] Prose and applicable tooling checks pass; release verification records the replacement candidate requirement.

## Resolution (2026-10-24)

done: corrected workflow query examples and tag-vocabulary discovery, preserving stored/environment context during temporary filtering. CLI behavior is unchanged. Replaced candidate preparation is complete; coordinated release gates remain in grey-matter69–71.

### Verification
1. ✓ Distributed workflow examples no longer invoke unsupported query --tag; scoped query examples use the actual context contract. — "The built 0.5.0 cd96042 CLI rejects query --tag with exit2. Source a5e26a2 removes all three invalid references; workflow tag vocabulary now comes from ticket frontmatter and temporary query scope uses a one-command environment override."
2. ✓ Isolated candidate CLI commands prove tag context filters queries and restoring context preserves the prior scope. — "Isolated Git-repo CLI replay passes: ink scope returns01+03, restored platform scope returns02+03, untagged03 remains included, query omits tags, and .tickets/.context bytes remain identical across overrides. Both no-override and existing-override cases are checked."
3. ✓ Fresh guidance/prose checks and applicable tooling tests pass, with a regenerated candidate tracked by grey-matter 69. — "37 Node tooling tests pass; YAML/core syntax validates4 skills; fresh prose review CLEAN after accounting for TKT_CONTEXT precedence; diff scope clean. Source bundle smoke builds3 closed artifacts. Replacement approved 0.5.0 candidate b2425c93f94a0b56a87cb6496ecbadaf381922bf is prepared from a5e26a2; grey-matter69 owns fresh product/native/install/CI evidence before staging."
