---
id: "289"
title: "Repair and smoke-test the first-run quick start"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "cargo test --test quick_start passes"
  - "capabilities scripting workflow selects at most one ID and full Rust tests pass"
  - "mise run check passes"
tags: ["stream:workflow", "kind:bug", "triage:ready-for-agent"]
assignee: "kimi"
---
# Repair and smoke-test the first-run quick start

## Intent source

Owner-approved audit on 2026-11-13 and implementation approval on 2026-11-15. Revalidated after pulling `9b0cb2f`: the README sequence fails before `new` without `.tickets/`, then fails at `close` with the default unchecked placeholder AC.

## Context

- `README.md` Quick Start and installation claims.
- `src/core/ticket.rs` default body template.
- `src/config.rs` default close policy.
- `src/commands/capabilities.rs` advertised scripting workflow.
- Integration tests exercise the built binary in isolated repositories.

Tracked by #291. Source documentation: `README.md`, Quick Start.

## What to build

A new user can copy the documented quick start into a fresh Git repository and finish with a valid closed ticket. The example initializes the store, supplies real intent and acceptance/validation content, and closes with checked acceptance and evidence. The machine-readable scripting example selects at most one ticket instead of claiming the entire frontier or passing several IDs to one `claim` invocation.

Add executable documentation-contract coverage so these examples cannot silently drift again. Preserve the newly added query match/tag/projection behavior.

## Acceptance criteria

- [x] The README quick start begins with initialization and its create/close sequence succeeds in an isolated Git repository.
- [x] The quick-start ticket has substantive body content and validation criteria and closes with acceptance and evidence.
- [x] The capabilities scripting example selects no more than one ready ID and does not use multi-argument `xargs tkt claim`.
- [x] Automated tests execute the quick-start path and validate the scripting contract.
- [x] The full maintainer gate passes.

## Out of scope

Changing default close policy, automatic repository initialization, or unrelated CLI behavior.

## Resolution (2026-11-15)

Made the README quick start executable with initialization, substantive content, validation and evidence; constrained scripted claiming to one ID and added isolated integration coverage.

### Verification
1. ✓ cargo test --test quick_start passes — "cargo test --test quick_start => 2 passed; fresh init/new/body/validation/close path and single-ready-ID recipe both passed"
2. ✓ capabilities scripting workflow selects at most one ID and full Rust tests pass — "capabilities workflow uses head -n 1 plus a nonempty-ID guard; cargo test --test capabilities => 3 passed; full cargo test completed successfully in mise gate"
3. ✓ mise run check passes — "mise run check retry completed all Rust/tooling suites successfully; commit 5e04025 pushed and installed binary reports 5e04025"
