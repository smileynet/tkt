# #145 update-check evidence

**Question:** Does `DO_NOT_TRACK=1` currently stop the update check from requesting
crates.io or refreshing its cache?

## Isolated fixture

Compiled the actual `src/update_check.rs` by absolute `#[path]` with `rustc 1.95.0`;
did not copy or alter its source. The small harness supplied only the `QUIET` and
`DRY_RUN` atomics normally provided by the CLI. A stub `dirs` rlib redirected
`dirs::config_dir()` to a per-case temporary directory from `TKT_TEST_CONFIG_DIR`.
The fake `curl.exe` wrote its arguments to a case-local marker and returned a fixed
crates.io-shaped response with `max_stable_version` `99.99.0`. It was placed beside
the harness executable: on Windows, executable search can find application/current
directories before PATH/system locations. A standalone `Command::new("curl")`
probe verified that the helper ran and wrote the marker before invoking the updater.
The harness used `CARGO_PKG_VERSION=0.1.0`; each case began with a stale cache
containing `0\n0.1.0\n`.

## Observations

| Environment | Fake curl called | Cache changed | Result |
| --- | --- | --- | --- |
| `DO_NOT_TRACK=1`; `CI` and `TKT_UPDATE_CHECK` unset | Yes | Yes; timestamp refreshed and version became `99.99.0` | Printed update notice |
| `TKT_UPDATE_CHECK=0`; `CI` and `DO_NOT_TRACK` unset | No | No | No notice |
| `CI=true`; `DO_NOT_TRACK` and `TKT_UPDATE_CHECK` unset | No | No | No notice |

This reproduces the gap in the current source: `DO_NOT_TRACK=1` does not prevent
the updater's request or cache write, while the existing update-disable and CI
guards do. The fake transport means these observations made no network request.

## Invalid initial attempt

An earlier harness placed the fake executable only in a PATH subdirectory. Windows
selected the system `curl.exe` instead; the marker was absent and that attempt must
not be treated as isolated evidence. The cache recorded version `0.4.0`, so a public
GET to the crates.io endpoint likely occurred. The command sent no request body; the
only explicit header was `User-Agent: tkt-update-check`. This was unintended. The
later, valid evidence above uses a fake executable beside the harness and a local
cache only.

## Regression-test seam

Exercise the public updater in a subprocess with a private config directory, a stale
cache fixture, cleared `CI`/`TKT_UPDATE_CHECK`, and `DO_NOT_TRACK=1`. Put a fake
`curl.exe` in the subprocess working directory (or executable directory), verify its
marker remains absent, and byte-compare the cache before and after. Separate cases
should prove `TKT_UPDATE_CHECK=0` and `CI` also suppress both effects. Keep the fake
transport and cache in a temporary directory; do not rely on PATH precedence alone
on Windows.

**Baseline scope:** Read-only source investigation plus isolated temporary harnesses.
At that point no Rust source, Cargo metadata, user cache, or project Git configuration
was changed. The baseline temporary harness and caches were removed to the OS Recycle
Bin.

## Implementation verification

The fixed source returns on exact `DO_NOT_TRACK=1` before it resolves or creates the
update-check cache path. `tests/update_privacy.rs` compiles the actual source by
absolute `#[path]`, with only a stub `dirs` crate and the CLI's two atomics supplied by
the fixture. No production dependency or source copy is involved.

Before any updater fixture invocation, the test runs `Command::new("curl")` with the
fixture directory as its working directory and PATH, then verifies the colocated fake
curl's marker contains `--preflight`. Each child process is started with
`DO_NOT_TRACK=1`. In the isolated, telemetry-free harness only, the two control cases
clear DNT internally to verify `TKT_UPDATE_CHECK=0` and `CI=true` independently. All
three cases verify the marker stays absent and the private config/cache directory is
not created. The fake response is local and no updater test can reach the network.

Observed checks:

- `cargo test --test update_privacy -- --nocapture` — PASS, 1 test passed.
- `cargo clippy --test update_privacy -- -D warnings` — PASS, zero warnings.
- `git diff --check` — PASS.
- `mise exec -- rustfmt --edition 2021 src/update_check.rs tests/update_privacy.rs` —
  completed for the changed Rust files.
- `cargo fmt --check` — FAIL due formatting differences in concurrent
  `src/renumber.rs` and `tests/rebase_refs.rs`; no unrelated files were formatted or
  changed as part of #145.

The focused test's `tempfile` fixture removes its harness, fake executables, marker,
and private cache when the test exits. The only permitted source changes for the fix
are `src/update_check.rs` and `tests/update_privacy.rs`.

## Final combined verification — 2026-11-03

`mise run check` with `RUST_TEST_THREADS=4` — PASS: 384 Rust tests, 42 Node
tests, zero failures; one existing Windows relative-link Node test SKIPPED.
Formatting passed and Clippy reported zero warnings. The general integration suite
passed 98 tests in 913.99 seconds; its unusual duration remains unexplained.
All 31 source/test identities in
[the source receipt](frontier-source-identities-2026-11-03.json) stayed unchanged
through this run. Independent critic accepted the final implementation, including
the added missing `ticket 998` assertion and explicit DNT on Git child processes.
Native Claude manifest validation and four-skill inventory passed; deploy manifest
check passed. These checks do not perform the directory safety/policy review.

## Installed verification — 2026-11-03

`cargo install --path . --locked` passed (49.84 s), and
`node tools/check-binary-fresh.mjs` reported fresh `768d392`, matching HEAD before
live checks. With `DO_NOT_TRACK=1`, installed strict validation and plan checks
passed with zero findings; query returned all 241 rows with required typed and
pending-blocker fields. A valid `new --dry-run -q -o json --validation ...` returned
one success object and preserved HEAD/worktree state. The initial preview probe
omitted the project's required validation criterion and was correctly rejected
with GateFailed/exit 1; that rejected input was not counted as a success.

`node tools/deploy.mjs` passed and installed four workflow skills plus the local
Antigravity pack. The generic missing-logo note is informational. The reproducible
`dist/agy-plugin` output is retained (338,341 bytes) for this deployment; the next
deployment replaces it. No rendered dashboard checks were performed.
