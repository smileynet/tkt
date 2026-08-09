---
id: "01"
title: "Set up cargo-dist and publish v0.1.0"
status: open
blocked_by: ["02", "03", "04", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15", "16", "17"]
---

# Set up cargo-dist and publish v0.1.0

## What to build

Configure cargo-dist to generate GitHub Actions CI that builds release binaries for all target platforms on tag push, then publish v0.1.0 to crates.io.

## Research findings (2026-08-09)

### cargo-dist setup

- **Current version**: v0.32.0 (May 2026). Rebranded from "cargo-dist" to "dist" at v0.24.0 but crate name unchanged.
- **Config file**: `dist-workspace.toml` (standalone, preferred since v0.23.0). tkt already has `[workspace.metadata.dist]` in Cargo.toml — `cargo dist init` will migrate.
- **Generated CI**: `.github/workflows/release.yml` — triggered by pushing a version tag.
- **"Always Be Initing"**: re-run `cargo dist init` after config changes or upgrades — it's idempotent.

### Recommended configuration for tkt

```toml
# dist-workspace.toml
[workspace]
members = ["cargo:."]

[dist]
cargo-dist-version = "0.32.0"
ci = "github"
installers = ["shell", "powershell"]
targets = [
  "x86_64-unknown-linux-gnu",
  "aarch64-unknown-linux-gnu",
  "x86_64-apple-darwin",
  "aarch64-apple-darwin",
  "x86_64-pc-windows-msvc",
]
```

**Installer choice**: Start with `shell` + `powershell` (curl|sh one-liners). Skip Homebrew for v0.1.0 (requires a separate tap repo). Add later if demand warrants.

**Targets**: 5 platforms (matches existing Cargo.toml metadata). glibc is fine since tkt requires `git` on PATH anyway (no musl benefit).

### crates.io readiness

- **Name check**: `tkt` — must verify availability at https://crates.io/crates/tkt (hyphen/underscore equivalence means `tkt` also blocks `tkt`... no conflict since no underscore variant). Fallback: `tkt-cli`.
- **Required fields**: ✅ license (MIT), ✅ description, ✅ repository. All present in Cargo.toml.
- **Pre-publish**: `cargo publish --dry-run` must pass. Verify `cargo package --list` doesn't include test fixtures or large assets.
- **Auth**: Need crates.io token from https://crates.io/settings/tokens, `cargo login`.

### Release workflow

1. `cargo install cargo-dist` (if not already installed)
2. `cargo dist init` — interactive wizard (choose targets, installers)
3. Remove existing `[workspace.metadata.dist]` from Cargo.toml (migrated to dist-workspace.toml)
4. `cargo dist plan` — verify release plan looks correct
5. Commit: `dist-workspace.toml` + `.github/workflows/release.yml`
6. Check crates.io name: `curl -s https://crates.io/api/v1/crates/tkt | jq .errors`
7. `cargo publish --dry-run` — verify package builds from .crate
8. `cargo publish` — upload to crates.io (permanent!)
9. `git tag v0.1.0 && git push && git push --tags` — triggers CI
10. Verify: GitHub Releases populated, installer scripts work

### Gotchas to watch for

- Version tag must match Cargo.toml version exactly (`v0.1.0` ↔ `version = "0.1.0"`)
- Commit Cargo.lock (required for binary crates)
- `.github/workflows/release.yml` runs plan step on PRs too (expected, catches issues early)
- Re-run `cargo dist init` after any future cargo-dist version upgrade

## Acceptance criteria

- [ ] `dist-workspace.toml` committed with correct targets and installers
- [ ] `.github/workflows/release.yml` generated and committed
- [ ] `cargo dist plan` produces correct release plan
- [ ] Crate name "tkt" available on crates.io (or fallback chosen)
- [ ] `cargo publish --dry-run` passes
- [ ] `cargo publish` succeeds (v0.1.0 live on crates.io)
- [ ] v0.1.0 tag pushed, GitHub Releases populated with 5 platform binaries
- [ ] Install script works: `curl -fsSL ... | sh` installs the binary
- [ ] `cargo install tkt` works from crates.io
