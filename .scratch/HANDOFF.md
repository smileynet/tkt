---
created_at: 2026-08-09T09:45:00-07:00
base_commit: cbfd69f
handoff_key: tkt-pre-release
---

# Handoff

## Objective
Ship tkt v0.1.0 — the only remaining ticket is #01 (cargo-dist + publish).

## Constraints
- `cargo fmt && cargo clippy --all-targets && cargo test` gate before every commit
- New mutation commands must route push through a push-gated path (GitTransaction or check pcfg.push_enabled)
- Codex on this machine requires `--dangerously-bypass-approvals-and-sandbox`
- `mise install` needed before `mise run release` (installs cargo-release + git-cliff)

## Prior Decisions
- Release stack: cargo-release + git-cliff + cargo-dist (not release-plz)
- Versioning: SemVer with CLI contract as API, v-prefix tags, stay at 0.x until stable
- Changelog: keepachangelog v2.0.0, six categories only, user-facing entries only
- Color: auto mode = on if tty, off if NO_COLOR or piped. No external crate.
- Config precedence: CLI flag > env var > project config > user config > default
- Skill distribution: spellbook owns skills, tkt owns `tkt capabilities` interface
- push.enabled=false is the mitigation for mutation latency (accepted as inherent)

## Current State
- 96 tests (48 unit + 48 integration), 0 clippy warnings
- Release toolchain configured: CHANGELOG.md, cliff.toml, release.toml, mise.toml
- Self-hosted GitHub Actions runner active: `randomserver-tkt` (systemd service)
- All Codex review findings (#50, #65) addressed
- Frontier: only #01 remains. #45 (self-update check) blocked by #01.

## Next Steps
1. `mise install` (gets cargo-release + git-cliff)
2. `cargo dist init` — generate `.github/workflows/release.yml`
3. `cargo dist plan` — verify release plan
4. Check crates.io: `curl -s https://crates.io/api/v1/crates/tkt | jq .errors`
5. `cargo publish --dry-run` → `cargo publish`
6. `git tag v0.1.0 && git push && git push --tags`
7. Verify GitHub Releases populated with binaries

## Fog
- crates.io name "tkt" availability (fallback: "tkt-cli")
- Whether cargo-dist's cross-compile matrix needs GitHub-hosted runners for macOS/Windows (self-hosted runner is Linux x64 only)

## Evidence
- `tkt ready` → 1 ticket (#01)
- `tkt blocked` → 1 ticket (#45, blocked by #01)
- `cargo test` → 96 pass, 0 fail
- Runner: `gh api repos/smileynet/tkt/actions/runners` → randomserver-tkt: online
