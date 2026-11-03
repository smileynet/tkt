# Quiet JSON output and query context evidence

## #143: quiet JSON for write-capable commands

The integration matrix in `tests/quiet_json.rs` invokes these successful writer
paths with `-q -o json`: `new`, `batch`, `claim`, `edit`, `close`, `cancel`,
`reopen` (after a cancellation setup), `renumber`, `context`, project `config
--set`, `init`, `lint`, `validate --fix`, `sync-plan --fix`, confirmed `migrate`,
`telemetry --enable`, `--disable`, `--clear`, and `rebase` against a local bare Git
remote. Every invocation is checked for exactly one stdout JSON object with `ok:true`
and a `result` field. Batch verifies one object for its multiple created tickets.
The test also checks `tkt --help` explains that `-o json` remains structured with
`--quiet`.

The same test covers quiet-JSON preview and no-op cases: dry-run `new`, `batch`,
`claim`, `edit`, `close`, `cancel`, `renumber`, `context`, project config set, and
`init`; unchanged and dry-run `lint`; dry-run `sync-plan --fix` and `validate --fix`;
no-remote `rebase`; and empty-source dry-run migration. The dry-run cases assert the
repository status stays unchanged; lint and plan previews byte-compare the files they
would otherwise rewrite. `doctor --fix` remains excluded from the writer list because
the inspected command currently emits a “not yet implemented” warning and performs
only the read-only doctor scan.

## #144: context filtering and bypass

`tests/query_context.rs` seeds one `ink` ticket, one differently tagged ticket, and
one untagged ticket. With `TKT_CONTEXT=ink`, query retains the `ink` and untagged
tickets, reports the filtering to stderr, and leaves JSON Lines stdout parseable.
`--all-context` returns all three and emits no context notice. The test also verifies
the JSON-mode stderr warning parses as one object with `warning.kind ==
"context_filter"` and names the bypass. With no active context, all tickets remain
and there is no notice. `query --help` documents the default filter and `--all-context`.

## Verification

- `cargo test --test quiet_json --test query_context -- --nocapture` — PASS: 3 quiet
  JSON tests and 2 query-context tests; the quiet JSON test binary reported 170.65 s.
- `mise exec -- rustfmt --check --edition 2021` on the changed CLI, command, and test
  files — PASS.
- `cargo clippy --test quiet_json --test query_context -- -D warnings` — PASS, zero
  warnings.
- `git diff --check` — PASS.
- Workspace `cargo fmt --check` — FAIL due formatting in concurrent #231/#232 files
  `src/commands/query.rs`, `tests/pending_blockers.rs`, and
  `tests/query_output_fields.rs`. The #143/#144 files pass the per-file rustfmt check;
  those concurrent files were left to their owner.

All integration child processes set `DO_NOT_TRACK=1`, user config is disabled, and
Git operations use disposable local repositories. Rebase uses a local bare remote;
the tests make no network requests. `tempfile` removes their fixtures on exit.

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
