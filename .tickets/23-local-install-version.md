---
id: "23"
title: "local install workflow and --version flag"
status: open
blocked_by: ["17"]
---

# Local install workflow and --version flag

## What to build

Ensure `cargo install --path .` works cleanly and add a `--version` / `-V` flag so users (and agents) can verify which build is running.

### Changes

1. **`--version` flag** — add `#[command(version)]` to the clap derive (reads from `Cargo.toml` automatically). Verify `tkt --version` outputs `tkt 0.1.0`.
2. **Install verification** — run `cargo install --path . --force`, confirm `tkt --version` matches, confirm `tkt ready` works from a different directory.
3. **Document in README** — the Install section already shows `cargo install --path .`. Verify it's accurate and add a verification step (`tkt --version`).

### Deletion test

Without --version, there's no way to confirm which binary is installed. Without verifying the install flow, adoption across projects is blocked.

## Acceptance criteria

- [ ] `tkt --version` prints version from Cargo.toml
- [ ] `tkt -V` works (short form)
- [ ] `cargo install --path .` succeeds and places binary on PATH
- [ ] Binary installed via cargo install matches `cargo build --release` output
- [ ] README Install section includes version check step
