---
created_at: 2026-08-18T14:25:00-07:00
base_commit: 6b9ae88
handoff_key: tkt-v020-prep
---

# Handoff

## Objective
Ship tkt v0.2.0 — the release ticket (96) is unblocked, changelog is written, all planned pre-release work is done.

## Constraints
- `close.allow_force = false` enforced globally via user config — no `--force` bypass anywhere
- All close operations require validation_criteria + evidence (user config cascade)
- `TKT_NO_USER_CONFIG=1` required in integration tests to prevent ambient config leaking
- CI workflows disabled (local gate only: `cargo fmt && cargo clippy --all-targets && cargo test`)
- cargo-dist release workflow still active on tag push (needed for binary releases)

## Prior Decisions
- tkt owns its own skills (not crew-research) — deployed via `tools/deploy-skills.sh`
- Errors on stderr as JSON envelope (last line), success on stdout — follows octo-cli pattern
- clispec.dev is inspiration, not a standard to conform to (9⭐, solo author)
- `NewTicketParams` struct replaces 8-param function (clippy-clean)
- Config cascade: CLI > env > project > user > default

## Current State
- 13 open tickets on frontier (5 medium, 8 low)
- Ticket 96 (Cut v0.2.0) is ready — all blockers done, changelog complete
- 56 integration tests pass, 0 clippy warnings
- All 10 repos across ~/code pass `tkt validate` with 0 findings
- README, AGENTS.md, SKILL.md all audited and current as of this commit

## Next Steps
1. **Cut v0.2.0** — ticket 96. Bump Cargo.toml, freeze changelog, tag, push tag (triggers cargo-dist)
2. **Post-release**: update `plugin.json` version, verify crates.io page
3. **After release**: frontier is 77 (migrate), 80 (context system), 82 (urgency scoring), 95 (env tags)

## Evidence
- `tkt ready` shows 13 tickets, 96 unblocked
- `tkt validate`: pass, 0 findings
- `tkt doctor ~/code`: 11 clean, 0 broken, 8 non-tkt
- `cargo clippy --all-targets`: 0 warnings
- `cargo test`: 56 passed
