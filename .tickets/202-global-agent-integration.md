---
id: "202"
title: "Design versioned embedded skills for binary-only installs"
status: in_progress
blocked_by: []
priority: high
validation_criteria:
  - "A concrete embedded-asset version and binary-only interface contract identifies supported harnesses and instruction/skill ownership"
  - "Dry-run, unmanaged conflicts, upgrade/rollback and duplicate handling are specified with testable fixtures"
  - "The design distinguishes shipped repository deployment from the remaining binary-only scope and links current source evidence"
tags: ["guidance", "agent-integration", "integrations", "kind:enhancement", "stream:guidance", "triage:ready-for-agent"]
assignee: "codex"
---

# Design versioned embedded skills for binary-only installs

## Native source-install observation — 2026-11-03

During ticket 247 deployment, local native plugin installation snapshotted the
checkout including ignored build artifacts: the Codex cache held 4,520,879,155
logical bytes (24,271 files), and the Claude cache held 4,506,902,767 logical bytes
(20,520 files). Both were installed once; explicit guarded deployment roots then
avoided an immediate second native snapshot while removing owned bare copies.
The 15 required workflow runtime files matched source. Claude actively reads the
checkout directly, while Codex uses its managed snapshot from the local marketplace.
Account for this observed source-checkout cost when designing embedded versioned
assets and lean export/install ownership; do not assume ignored build data is
excluded by a native installer. This is design input, not an implementation or
an instruction to unpark this deliberately deferred ticket.

## Owner decision — 2026-11-03

Confirmed in chat: **Design embedded versioned skill assets**. This supersedes conflicting
requirements and the earlier dated decision gate; it does not claim implementation.
See [the authoritative record](../docs/decision-record-2026-11-03.md).

## What to build

Design versioned skill/instruction assets embedded in the binary and a binary-only export/install interface. Specify owned replacement, unmanaged conflicts, supported harness roots, project instructions versus global skills, dry-run, upgrade/rollback and duplicate guards. Reconcile the current deploy engine before choosing final flags; no source checkout should be required by the designed binary-only path.

## Context

- Read `src/commands/init.rs` for current project-local targets and markers.
- Read `tools/deploy-skills.sh` for current local symlink behavior.
- Read `docs/agents/agent-guidance-surfaces.md`; a CLI behavior change still requires all
  listed guidance surfaces to remain synchronized.
- Verify each harness path against its current official documentation before adding it
  to the registry. A claimed but unverified target is worse than an explicit skip.

## Acceptance criteria

- [ ] A concrete embedded-asset version and binary-only interface contract identifies supported harnesses and instruction/skill ownership
- [ ] Dry-run, unmanaged conflicts, upgrade/rollback and duplicate handling are specified with testable fixtures
- [ ] The design distinguishes shipped repository deployment from the remaining binary-only scope and links current source evidence

## Out of scope

- Teaching every harness to consume skills it does not natively support
- Changing the ticket workflow itself

## Rescope (2026-10-29)

Most of this shipped in 0.5.0 through the shared deploy engine (`tools/deploy.mjs`,
vendored from grey-matter; `tools/deploy-skills.sh` now forwards to it):

- `--dry-run` plans every action without writing.
- An ownership manifest records what the deployer installed; unowned same-named
  directories are refused, not deleted.
- Missing managed roots are created; OpenCode is covered (grey-matter's deploy tests
  include an OpenCode-only home, served through `~/.claude/skills` or its own root).
- Claude, Codex and Antigravity users can install the plugins without a checkout.

Remaining gap: a user who installs only the binary (`cargo install tkt`) and uses
OpenCode or Kiro still needs a repository checkout to get the four workflow skills.
Narrow the work to that: package the skills with the binary and give it a deploy
command. Check off the delivered criteria above with evidence when this ticket is
worked.

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:needs-info`; stream `integrations`.

Repository deploy already exists; remaining binary-only skill delivery is excluded by Cargo.toml include inventory. Decide embedded-assets/deploy command before implementation; verify harness paths when worked.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.

## Planning gate

Resolved by the owner's 2026-11-03 chat submission: **Design embedded versioned skill assets**.
See [the decision record](../docs/decision-record-2026-11-03.md). Backlog remains deliberately deferred.


## Reactivation — 2026-11-03

The owner requested review of all parked work and execution of actionable tickets. This supersedes earlier blanket deferral. See [the parked-work review](../docs/parked-review-2026-11-03.md) for current scope and sequencing. Independent critic review is required before closure.
