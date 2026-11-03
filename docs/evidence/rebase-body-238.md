# #238 rebase body reference evidence

## Baseline

Captured before changing product source. The fixture in [`tests/rebase_refs.rs`](../../tests/rebase_refs.rs) follows the bare-remote/two-clone collision setup in `tests/integration.rs::test_rebase_resolves_id_collision`. It asserts both colliding IDs are present in `tkt query -o json`, then runs the installed build output at `target/debug/tkt.exe` through Cargo's integration-test binary. Child probes set `DO_NOT_TRACK=1`, `TKT_NO_USER_CONFIG=1`, and `TKT_UPDATE_CHECK=0`. `cargo test --test rebase_refs -- --nocapture` passed (1 test, 0 failed).

The gate-built working-tree binary reports `tkt 0.4.0 (3868d0f)`. Its version stamp is not asserted to identify the exact source commit. At capture, `HEAD` was `96d19718d202a708475df7b44abe1ec7fe44574b`; the source object IDs were `82d289debcc77057bc4dd8828d77455498d64056` (`src/commands/rebase.rs`), `414a3eceb3d5bb291ff3115bc9cfb53e35884f9b` (`src/renumber.rs`), and `88c627e52e83ae16c5f386bc17f48ace01b3a0fe` (`src/findings.rs`).

## Observed collision

Clone A pushed upstream `02-alpha`; stale clone B held its own `02-beta` and `03-gamma`, whose `blocked_by` points to `02`. Beta body cited `#02`, `ticket 02`, and unrelated `#999`; Gamma body cited `#02` and `ticket 02`; upstream Alpha body also cited `#02` and `ticket 02`.

`tkt rebase` exited 0 and printed:

```text
Renumbered 1 ticket(s):
  02 → 04 (beta)
  1 blocked_by reference(s) updated
```

Beta's ID and filename changed to `04-beta.md`; Gamma's frontmatter changed to `blocked_by: ["04"]`. Neither local body changed. Git blob IDs and body lengths:

| File/body | Before | After | Body bytes |
|---|---|---|---:|
| Beta full ticket | `e88433d33bc7c4b94503b3213471115d47af8b42` | `218230e13bcd0bdad85d03e0f050ee78d4dd24f2` | — |
| Beta body | `77a90fb80998a88c80e76631562650e49e703bf2` | same | 87 / 87 |
| Gamma full ticket | `241f8a34b0e51e866c1e3eab200a7fccafb4d454` | `73144063f6bc573654ac59059c5480ff1ff94bf0` | — |
| Gamma body | `5bdcd05e62521d2bdf37da93ae01ceb53ada4e7a` | same | 51 / 51 |
| Upstream Alpha ticket | `ed9c6bf5177f98746927db10ce82f91d4e47e703` | same | unchanged |

The fixture asserts local body byte equality and unchanged upstream ticket content. Existing `tkt validate -o json` exited 0 with only the unrelated seed's `missing-resolution` warning; it did not flag Beta's `#999` reference. The run therefore confirms both AC gaps in this corpus: the rebase output omitted affected body citations, and validation ignored a dangling numeric citation.

## Bounded behavior recommendation

Use unqualified `#NNN` and `ticket NNN` for local ticket references. Qualify an external GitHub issue as `owner/repo#NNN`, or use a Markdown link to the external issue. Report affected local files and the old-to-new mapping when a body contains a recognized citation to a renumbered ID. Preserve body bytes: prose is user-owned, and an unqualified `#02` may refer to the upstream Alpha that retains ID 02, not the local Beta that moved to 04. Validation warns on unqualified `#NNN` and `ticket NNN` references with no matching corpus ID, while suppressing code samples, URLs, external Markdown link labels, and repository-qualified citations such as `owner/repo#NN`. Such matches are heuristic warnings, not proof that prose means a local ticket; the diagnostic tells the author to verify ambiguous references.

The fixture proves the ambiguity concretely: the same `#02`/`ticket 02` string occurs in the moved local ticket, its dependent, and the upstream ticket that keeps ID 02. It does not establish whether every existing production-board prose citation is intended to be local, external, or historical. A corpus-wide strict-validation pass is still needed after implementation to identify and review false positives.

## Implementation verification

The source change makes rebase list affected body citations with file, line, displayed text, and old-to-new mapping in both dry-run and real runs; it preserves the citation text. The regression fixture checks those lines for the moved Beta, dependent Gamma, and unchanged Alpha, and asserts dry-run leaves local bytes untouched and Beta/Gamma body bytes remain equal to their pre-rebase Git blob hashes after a real run. It also checks validator warnings for dangling `#999` and `ticket 998`, and confirms the sample inline code, fenced code, URL fragments, external Markdown link label, repository-qualified reference, and numeric Markdown heading do not generate citation warnings.

Focused command: `cargo test --test rebase_refs -- --nocapture` — **passed**, 1 test, 0 failed. The isolated post-change `tkt validate -o json` exited 0 and reported `#999` in `04-beta.md` plus the seed's pre-existing `missing-resolution` warning. This is a fixture-level check only; the production ticket corpus still needs review under strict validation after installing the updated binary.

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
