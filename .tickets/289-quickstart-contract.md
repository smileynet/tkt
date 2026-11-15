---
id: "289"
title: "Repair and smoke-test the first-run quick start"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "cargo test --test quick_start passes"
  - "capabilities scripting workflow selects at most one ID and full Rust tests pass"
  - "mise run check passes"
tags: ["stream:workflow", "kind:bug", "triage:ready-for-agent"]
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

## What to build

A new user can copy the documented quick start into a fresh Git repository and finish with a valid closed ticket. The example initializes the store, supplies real intent and acceptance/validation content, and closes with checked acceptance and evidence. The machine-readable scripting example selects at most one ticket instead of claiming the entire frontier or passing several IDs to one `claim` invocation.

Add executable documentation-contract coverage so these examples cannot silently drift again. Preserve the newly added query match/tag/projection behavior.

## Acceptance criteria

- [ ] The README quick start begins with initialization and its create/close sequence succeeds in an isolated Git repository.
- [ ] The quick-start ticket has substantive body content and validation criteria and closes with acceptance and evidence.
- [ ] The capabilities scripting example selects no more than one ready ID and does not use multi-argument `xargs tkt claim`.
- [ ] Automated tests execute the quick-start path and validate the scripting contract.
- [ ] The full maintainer gate passes.

## Out of scope

Changing default close policy, automatic repository initialization, or unrelated CLI behavior.
