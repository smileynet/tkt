# Query output fields and pending blockers (#231, #232)

## Output contract

`query` remains JSON Lines and keeps the full historical `blocked_by` list. It now adds `pending_blockers`, computed as listed dependencies whose ticket is not `done`; missing IDs, `open`, `in_progress`, `backlog`, and `cancelled` IDs remain pending. This is diagnostic output only and does not alter dependency edges.

`ready --json` also includes `pending_blockers`. Its rows are drawn from the frontier, so this list is empty for each ready ticket; `blocked_by` still records completed historical prerequisites. Human `blocked` output continues to show only unfinished blockers, including a `not found` marker for missing tickets.

`query` now includes typed `tags`, `validation_criteria`, and effective `requires` arrays, alongside its existing typed fields, `agents`, and `assignee`. Explicit `env: either` remains visible; valid empty `spec:` and `parent:` values are emitted. Optional fields absent from the ticket remain omitted, preserving their prior shape. Unknown or unparsed stored fields are preserved under a `frontmatter` object as trimmed raw YAML value strings. That namespace also protects computed top-level fields from user keys such as `pending_blockers` or `frontmatter`.

## Focused verification

Command: `cargo test --test query_output_fields --test pending_blockers --test rebase_refs --test query_context` — **passed**, 5 tests, 0 failed. The fixtures cover all recognized typed frontmatter fields, explicit default/empty optional fields, raw unknown fields and reserved-name collisions; done, unfinished, cancelled, and missing blockers; ready JSON and human blocked output; context filtering and `--all-context`; and JSON warning output on stderr.

The rebase fixture used by #238 also verifies both dangling body citation forms: plain `#999` and `ticket 998` warn, while the Markdown external link label `[#999](https://github.com/owner/repo/issues/999)` does not add a local warning. All Git child processes in the fixture set `DO_NOT_TRACK=1`.

The repository-wide combined gate is running separately; its result is recorded by the root task owner. These focused results do not replace that gate or the production-corpus strict validation pass.

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
