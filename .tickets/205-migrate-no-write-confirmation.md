---
id: "205"
title: "Keep migration confirmation without writing during dry runs"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "tkt migrate --dry-run makes no filesystem changes"
  - "a reviewed migration requires explicit non-dry-run confirmation before applying"
tags: ["migration", "release"]
---

# Keep migration confirmation without writing during dry runs

## What to build

Replace the `.migrate-reviewed` marker written by `tkt migrate --dry-run` with an
explicit confirmation path that preserves two guarantees: previewing makes no
filesystem changes, and applying a migration still requires an intentional action
after the plan has been reviewed. The confirmation must be observable to a fresh
agent and must not rely on untracked mutable state left by a dry run.

Source: #136 dry-run remediation, discovered 2026-10-19. The existing marker is a
deliberate safety gate, so simply removing it would weaken migration safety.

## Context

- Read `src/cli.rs` for the migrate argument surface and `src/commands/migrate.rs`
  for the current marker protocol.
- Update every applicable agent/user guidance surface if a confirmation flag is added.
- Preserve the migration plan output and orphan-dependency warning behavior.

## Out of scope

- Changing the tk-to-tkt conversion mapping
- Allowing an unreviewed migration to apply automatically

## Acceptance criteria

- [x] `tkt migrate --dry-run` leaves the ticket directory and repository status
      byte-identical
- [x] Applying a migration requires an explicit confirmation after a dry-run plan has
      been reviewed, without a filesystem marker
- [x] Integration tests cover no-write preview, unconfirmed apply refusal, and
      confirmed application

## Resolution (2026-10-19)

Replaced the filesystem marker with an explicit --confirm apply step after reviewing the no-write migration preview.

### Verification
1. ✓ tkt migrate --dry-run makes no filesystem changes — "Verified preview preserves ticket bytes and git status, and creates no marker or backup; the regression test covers it."
2. ✓ a reviewed migration requires explicit non-dry-run confirmation before applying — "Verified unconfirmed apply fails, --confirm applies the reviewed plan, and integration coverage passes."
