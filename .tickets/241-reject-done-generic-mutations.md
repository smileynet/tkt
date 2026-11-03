---
id: "241"
title: "Reject done status outside the close lifecycle"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "Generic status mutations cannot create or set done; close remains the evidence-gated route"
  - "Integration fixtures prove failed mutations preserve ticket bytes and Git state"
tags: ["cli", "kind:bug", "triage:ready-for-agent", "stream:workflow"]
---

# Reject done status outside the close lifecycle

## What to build

Reject `done` in generic creation/status-edit routes. Only `tkt close` may reach
done, after its resolution, checked-AC and validation-evidence gates succeed.
Preserve supported open/backlog/in_progress behavior and global dry-run semantics.

## Context and confirmed reproduction

Found while implementing #138 on 2026-11-02. AGENTS.md defines the close-only
lifecycle; the supported capabilities schema follows that contract. Runtime
`src/commands/edit.rs` instead parses every Status and writes it directly.
`new` and `batch` call validate_status, which currently accepts done as well;
verify those creation routes before changing them.

Confirmed in a new, local-only Git fixture with push.enabled=false and a ticket
containing an unchecked AC and no Resolution: freshly built target/debug/tkt
0.4.0 (d191fbe), `edit 01 --status done` exited 0 and printed
`edited 01 fixture (status)`. The file then had status done, the AC stayed
unchecked and no Resolution section existed. This was an isolated fixture,
not a project-ticket closure. Original fixture bytes are described here so the
temporary fixture can be discarded after this evidence is promoted.

Read src/commands/{edit,new,batch,close}.rs, src/core/validate.rs and the
frontmatter/closure contract. This is independent of #154's detection of hand
edits and #227's cancellation lifecycle; neither should be weakened or duplicated.

## Acceptance criteria

- [x] Reproduce edit/new/batch behavior independently and classify each confirmed, rejected or obsolete before changing code.
- [x] Generic status mutations reject done with a structured Validation error and direct the caller to close.
- [x] A rejected invocation leaves ticket bytes, staging and Git HEAD unchanged, including dry-run cases.
- [x] Normal close still enforces its gates and writes the Resolution; other supported statuses retain compatibility.
- [x] Public CLI integration fixtures and the full local gate pass.

## Resolution (2026-11-03)

New/batch/edit reject done before mutation with Validation and close hint; close gates and compatible statuses retained. Independent critic accepted final guidance/capabilities. docs/evidence/lifecycle-identifiers-config-241-164-140.md.

### Verification
1. ✓ Generic status mutations cannot create or set done; close remains the evidence-gated route — "Three lifecycle integration tests prove generic done rejected and close gates remain; baseline confirmed all three routes accepted done."
2. ✓ Integration fixtures prove failed mutations preserve ticket bytes and Git state — "Normal and dry-run rejection tests preserve ticket bytes, index status and HEAD; full gate322Rust42Nodepass one existing skip."
