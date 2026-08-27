---
created_at: 2026-08-27T10:02:18-07:00
base_commit: 00e717a
handoff_key: tkt-v031-bugfix
---

# Handoff

## Objective
Ship v0.3.1 — close the data-loss/contract bugs blocking the release (#146 is the release-cut ticket, blocked by 131-141 + 127).

## Constraints
- Windows: mise shims broken → use `D:\dev-tools\cargo\bin\cargo.exe` and toolchain rustfmt directly (AGENTS.md:145).
- `git commit -F <file>` for multi-line messages (PowerShell mangles `-m`).
- **Stale-binary trap (hit 3×):** the installed `tkt` drifts from HEAD after commits. Validate via `cargo test` integration tests (`tkt_bin()` = auto-rebuilt `target/` binary), NOT the PATH binary. See #153.
- Project enforces `require_validation_criteria` at `tkt new` and one `--evidence` per criterion at close.
- Never fabricate resolutions; never hand-flip `status: done` (enforced now — see #154).

## Prior Decisions
- Tags shipped in v0.3.0 (`--tags`, plural) — do NOT reimplement.
- #155 decided hook-enforcement policy: warn-default, block opt-in via `[hooks]` config, logging opt-in (off by default). Client hooks are advisory; CI `validate --strict` is the only real gate.

## Current State
Run `tkt ready` for the frontier. Closed this session: #131 (block-style/bare blocked_by), #132 (BOM/comment/space-colon parse tolerance), #154 (validate flags hand-flipped done + guidance), #155 (hook spike → decision record). Nothing left mid-flight — clean tree at 00e717a.

## Next Steps
- Continue the #146 blocker set: #133 (yaml_scalar_unescape corrupts plain scalars), #134, #135, #136 are the next HIGH-priority parser/contract fixes on the frontier.
- #153 (bumped HIGH): implement the stale-binary practice-change — the recurring trap.
- Follow the propose → dispatch research → update ticket → implement → verify-via-cargo-test → close loop that worked for 131/132/154.

## Fog
- #160: 27 legacy tickets (#02-#35) now warn `missing-resolution` (side-effect of #154 firing on pre-#154 closes). Policy undecided: grandfather by close-date vs accept as documented. Do NOT backfill (fabrication).
- #157/#158/#159 (hook impl follow-ups) not yet scoped for sequencing against the release.

## Evidence
- Frontier: `tkt ready`. Health: `tkt validate --brief` (27 expected legacy warnings, else pass).
- Hook research preserved: `.scratch/subagent-raw/{agent-hook-mechanisms,beads-hooks,git-hook-install-patterns,guardrail-warn-vs-block,codex-applypatch-hook,tkt-init-git-review}.md` (referenced by #155/#157/#158/#159).

## Recommended Updates
- [ ] #153 is the highest-leverage process fix — prose constraint (AGENTS.md:148) failed 3×; needs the mechanical `cargo test` practice-change.
- [ ] AGENTS.md is at the 150-line ceiling — next Constraint addition should trigger a trim (`/agents-md-authoring`).
