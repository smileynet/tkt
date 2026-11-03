---
id: "157"
title: "tkt hooks subcommand + pre-commit installer (warn-default, config block/log)"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "The installer is explicitly optional/advisory and preserves foreign hooks and managed ownership"
  - "Mutation hook bypass and standing verification policy remain intact; no implicit global or blocking hook is installed"
  - "Portable invocation, failure recovery and project-scoped configuration are documented and verified"
tags: ["contract", "integrations", "kind:enhancement", "stream:guidance", "triage:ready-for-agent"]
---

# tkt hooks subcommand + pre-commit installer (warn-default, config block/log)

## Owner decision — 2026-11-03

Confirmed in chat: **Keep hooks optional and advisory**. This supersedes conflicting
requirements and the earlier dated decision gate; it does not claim implementation.
See [the authoritative record](../docs/decision-record-2026-11-03.md).

## What to build

Design the optional project hook installer as advisory. Preserve existing hooks and foreign content, the current mutation hook-bypass policy, and local verification policy. Installation is explicit and project-scoped; do not add a global hook or an implicit blocking gate. Document portable invocation, ownership, backup/recovery and interaction with normal Git versus tkt commits.

## Context

- **Relevant files:** `src/cli.rs` (Hooks subcommand), `src/git.rs` (add `--git-common-dir` helper), `src/config.rs` (`[hooks]` section), `src/telemetry.rs` (log sink pattern)
- **Zero new deps** — const-string shim + `std::fs::write` + `git rev-parse`
- **Windows:** emit LF-only shim, pin `.gitattributes eol=lf`, copy (not symlink), keep logic in the binary
- From #155 decision record; research in `.scratch/subagent-raw/git-hook-install-patterns.md`

## Acceptance criteria

- [ ] The installer is explicitly optional/advisory and preserves foreign hooks and managed ownership
- [ ] Mutation hook bypass and standing verification policy remain intact; no implicit global or blocking hook is installed
- [ ] Portable invocation, failure recovery and project-scoped configuration are documented and verified

## Out of scope

- Per-agent write-nudge (#158)
- CI gate docs (#159)

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `integrations`.

Deferred opt-in product hook installer. Reconcile with #235 skipping git hooks and standing guidance against installing verification hooks; do not silently install hooks during this triage. Historical scratch research is not a dependency.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Resolved by the owner's 2026-11-03 chat submission: **Keep hooks optional and advisory**.
See [the decision record](../docs/decision-record-2026-11-03.md). Backlog remains deliberately deferred.


## Reactivation — 2026-11-03

The owner requested review of all parked work and execution of actionable tickets. This supersedes earlier blanket deferral. See [the parked-work review](../docs/parked-review-2026-11-03.md) for current scope and sequencing. Independent critic review is required before closure.
