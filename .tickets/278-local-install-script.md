---
id: "278"
title: "One-command local install: build + stable install + freshness check"
status: in_progress
blocked_by: []
validation_criteria:
  - "tools/install-local.sh builds and installs to --root or default HOME/.local"
  - "install target is outside toolchain-managed dirs"
  - "idempotent rerun; post-install --version embeds HEAD hash; loud failure on mismatch"
  - "plugin-maintenance.md documents the script and rationale"
tags: ["type:chore"]
---
## What to build

A one-command local install script, `tools/install-local.sh`, that builds the
release binary and installs `tkt` to a configurable, stable root outside any
toolchain-managed directory, then verifies the install.

**Why:** on 2026-11-08 a session found the installed `tkt.exe` was a stale build
living *inside* a mise-managed Rust toolchain directory
(`%LOCALAPPDATA%\mise\installs\rust\1.95.0\`), which a toolchain upgrade or
`mise prune` can wipe. Refreshing it required hand-copying a binary. The
documented `cargo install --path .` targets whatever `CARGO_HOME` a version
manager happens to own — not a location the user controls.

**Behavior:**

- `bash tools/install-local.sh [--root DIR]` (default root: `$HOME/.local`,
  binary lands at `<root>/bin/tkt[.exe]`).
- Builds `cargo build --release` (reuses target cache; fast when fresh).
- Copies the release binary into `<root>/bin/`, creating it if needed.
- Verifies the installed binary runs and its `--version` git hash equals HEAD;
  prints the PATH to add when `<root>/bin` is not on PATH.
- Idempotent; safe to re-run.

**Docs:** update `docs/agents/plugin-maintenance.md` — install section documents
the stable-location pattern (why not the version-manager toolchain dir) and the
script as the canonical local-refresh path.

## Acceptance criteria

- [ ] `tools/install-local.sh` exists; builds and installs to `--root` or default `$HOME/.local`
- [ ] Install target is outside toolchain-managed directories (mise/rustup homes never written)
- [ ] Re-running is idempotent; post-install `--version` embeds current HEAD hash and script fails loudly on mismatch
- [ ] `docs/agents/plugin-maintenance.md` documents the script and the stable-location rationale
