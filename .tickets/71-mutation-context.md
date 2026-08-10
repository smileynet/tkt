---
id: "71"
title: "Introduce MutationContext for existing-ticket mutation workflow"
status: open
blocked_by: ["70"]
priority: high
---

# Introduce MutationContext for existing-ticket mutation workflow

## What to build

Introduce a `MutationContext` type that encapsulates the shared lifecycle for commands that mutate existing tickets (claim, close, edit, renumber).

### Intent

Mutation commands repeat an implicit protocol: resolve .tickets/ → resolve repo root → load project config → detect remote → fetch → load corpus → find ticket → check remote status → mutate → stage → commit → push (respecting push.enabled). This protocol is currently spread across `preflight_mutation()`, `check_remote_status()`, `commit_and_publish()`, and inline command code. The AGENTS.md constraint "new mutation commands MUST route push through a push-gated path" is enforced by code review, not architecture. MutationContext makes this structural.

### Context

- `preflight_mutation()` in cli.rs returns `(PathBuf, bool, Vec<Ticket>)` — a tuple, not a named type
- `commit_and_publish()` re-reads project config to check push.enabled (redundant load)
- `GitTransaction` already exists for allocation (new/batch) but NOT for existing-ticket mutation
- The project convention (AGENTS.md Constraints) says push must be gated — this is currently a review-only guarantee
- Blocked by #70 because it depends on command modules existing to consume it

### Desired outcome

After this work:
- `MutationContext::open()` resolves repo, .tickets/, config, remote state, fetches, and loads corpus — one call
- `ctx.find_ticket(id)` returns the ticket or a domain error
- `ctx.remote_status(ticket)` returns typed remote status
- `ctx.publish(paths, message)` stages, commits, and pushes — respecting push.enabled by construction
- Mutation commands become: open context → get ticket → domain logic → ctx.publish()
- The push-gating constraint is impossible to violate if you go through MutationContext

### How to validate

1. `cargo test` — all tests pass
2. `grep -r "git::push\|push_with_retry" src/commands/` — no direct push calls in command modules (all go through ctx.publish or GitTransaction)
3. Mutation commands (claim, close, edit, renumber) all use MutationContext
4. Adding a hypothetical new mutation command requires only: args → ctx.open() → logic → ctx.publish()
5. The type makes incorrect ordering impossible (can't publish before loading corpus, can't find ticket before fetch)

## Acceptance criteria

- [ ] `src/mutation.rs` (or `src/commands/common.rs`) exports `MutationContext`
- [ ] `MutationContext::open()` handles: tickets dir, repo root, config, remote detection, fetch, corpus load
- [ ] `ctx.find_ticket(id)` with proper domain error
- [ ] `ctx.publish(paths, message)` respects push.enabled
- [ ] `cmd_claim`, `cmd_close`, `cmd_edit`, `cmd_renumber` migrated to use it
- [ ] No direct `git::push` calls in command modules (push-gating enforced structurally)
- [ ] All tests pass, no behavioral change
