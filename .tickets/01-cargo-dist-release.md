---
id: "01"
title: "Set up cargo-dist and publish v0.1.0"
status: open
blocked_by: []
priority: high
---

# Set up cargo-dist and publish v0.1.0

## What to build

Configure cargo-dist to generate GitHub Actions CI that builds release binaries for all 5 target platforms on tag push, then publish v0.1.0 to crates.io.

## Steps

1. `cargo install cargo-dist`
2. `cargo dist init` — generates `.github/workflows/release.yml`
3. Verify the workflow looks correct (targets match Cargo.toml config)
4. Check crates.io: is the name "tkt" available? If not, use "tkt-cli"
5. Commit workflow, tag: `git tag v0.1.0 && git push --tags`
6. Wait for CI to build all platforms
7. Verify: GitHub Releases has binaries for linux/mac/windows + install scripts
8. `cargo publish` — publish to crates.io
9. Test: `cargo install tkt` on a clean env, run against crew-research .tickets/

## Acceptance criteria

- [ ] `.github/workflows/release.yml` committed and working
- [ ] v0.1.0 tag pushed, GitHub Releases populated with 5 platform binaries
- [ ] Install script works: `curl -fsSL ... | sh` installs the binary
- [ ] `cargo install tkt` works from crates.io
- [ ] Installed binary produces identical output to local build on crew-research corpus
