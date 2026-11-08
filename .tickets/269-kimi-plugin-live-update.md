---
id: "269"
title: "Measure tkt Kimi plugin update behavior"
status: open
blocked_by: []
validation_criteria:
  - "test -s docs/evidence/kimi-plugin-live-update.md"
  - "node --test tools/kimi-plugin.test.mjs"
tags: ["tooling", "gate:evidence"]
assignee: "kimi"
---
# Intent source

Sam requested follow-up tickets for remaining Kimi plugin limitations on 2026-11-08. Closed ticket 267 and `docs/agents/plugin-maintenance.md` explicitly leave live update timing unverified.

# What to learn and deliver

Observe the personal-plugin update lifecycle for tkt: initial registration and installation, a content change, re-registration, and the point at which a new chat receives changed skill bytes. Distinguish same-version re-registration from the supported Cargo/manifests version-change path and record required reload or restart behavior.

Use a disposable plugin copy for experiments. Do not hand-edit release versions, publish a release, or treat registration read-back as proof of an updated active session.

# Evidence to retain

Add `docs/evidence/kimi-plugin-live-update.md` with versions, operations, loaded byte comparisons, session state, and remaining uncertainty. Update `docs/agents/plugin-maintenance.md` with the verified procedure.

## Acceptance criteria

- [x] Same-version and supported changed-version behavior are measured separately.
- [ ] A live skill load proves which package bytes the new session received.
- [x] Documentation states observed timing without claiming instantaneous updates.
- [x] Existing Kimi package tests still pass.

## Note — 2026-11-08

By: anonymous

<!-- tkt-note:v1 8711-1794173332418443000 -->
2026-11-08 disposable probe recorded in docs/evidence/kimi-plugin-live-update.md: same-version registration preserved installed/current-session bytes; +local registration exposed Update. Update was not clicked and no fresh chat was observed, so AC2 remains unchecked. Registration restored to maintained dist/kimi-plugin at 0.4.0.
