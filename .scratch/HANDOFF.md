---
created_at: 2026-08-18T14:31:55-07:00
base_commit: 87a1e39
handoff_key: tkt-post-release
---

# Handoff

## Objective
v0.2.0 shipped. Next phase: work the remaining frontier (feature tickets) toward v0.3.0.

## Constraints
- User config enforces: `allow_force=false`, `require_validation_criteria=true`, `require_validation_evidence=true` globally
- `TKT_NO_USER_CONFIG=1` required in all integration test child processes
- CI disabled (local gate only); release workflow triggers on tag push (cargo-dist builds binaries)
- tkt owns its own skills — `bash tools/deploy-skills.sh` after changes to `skills/` or `steering/`
- AGENTS.md is the authority for project conventions; SKILL.md for agent activation

## Prior Decisions
- Errors on stderr (octo-cli pattern, not stdout) — `DomainError { kind, message, hint }`
- Config cascade: CLI > env > project > user > default (`src/config.rs`)
- `NewTicketParams` struct for ticket creation (eliminates clippy warning)
- agentskills.io: `name` not `title` in SKILL.md; `plugin.json` has no `skills` array
- clispec.dev is inspiration only (9⭐ solo author) — not a conformance target

## Current State
- v0.2.0 tagged and pushed (`496e9e8`), cargo-dist building binaries
- 12 open tickets on frontier: 77, 80, 82, 95 (medium); 79, 83, 86-90, 93 (low)
- 4 backlogged: 66, 69, 94, 99
- All repos pass `tkt validate` and `tkt doctor ~/code` clean (11 pass, 8 non-tkt flagged)
- 56 tests, 0 clippy warnings, README/AGENTS.md/SKILL.md audited this session

## Next Steps
1. Verify cargo-dist built binaries and GitHub Release created (`gh release view v0.2.0`)
2. Verify crates.io publish (`cargo search tkt` shows 0.2.0)
3. Pick next frontier work — recommended: 77 (migrate) or 82 (urgency scoring)
4. Consider backlog promotion: 99 (configurable update interval) is low-effort

## Fog
- v0.3.0 scope undefined — no plan document yet. Tickets 77/80/82/95 are candidates but priority not agreed.
- `tkt migrate` (77) has a migration-assist skill but no implementation yet — foreign schema detection is research-dependent.

## Evidence
- `tkt --version`: `tkt 0.2.0 (496e9e8)`
- `git tag -l v0.2.0`: exists, pushed to origin
- `tkt ready`: 12 tickets (5 medium, 7 low)
- `tkt doctor ~/code`: 11 clean, 0 broken
