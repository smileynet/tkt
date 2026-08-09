---
created_at: 2026-08-09T09:20:00-07:00
base_commit: bd9f26c
handoff_key: tkt-pre-release
---

# Handoff

## Objective
tkt v0.1.0 release. All features complete, review findings addressed, release toolchain configured. Only `cargo dist init` + publish remains.

## Current State
- 96 tests (48 unit + 48 integration), clippy 0 warnings, fmt clean
- Release toolchain: CHANGELOG.md, cliff.toml, release.toml, mise.toml with [tools]
- Self-hosted GitHub Actions runner configured (`randomserver-tkt`, systemd service)
- All Codex review findings (#65) addressed — 8 fixed, 1 rejected, 1 deferred

## Next Steps
1. `cargo install cargo-release git-cliff` (or `mise install`)
2. `cargo dist init` — generate `.github/workflows/release.yml`
3. Check crates.io: `curl -s https://crates.io/api/v1/crates/tkt | jq .errors`
4. `cargo publish --dry-run` then `cargo publish`
5. `git tag v0.1.0 && git push && git push --tags`
6. Verify GitHub Releases populated

## Frontier
```
tkt ready → 01  Set up cargo-dist and publish v0.1.0
tkt blocked → 45  self-update check (blocked by 01)
```

## Fog
- crates.io name "tkt" availability (fallback: "tkt-cli")
- Whether self-hosted runner can also handle cargo-dist's cross-compile matrix (likely needs GitHub-hosted for macOS/Windows)
