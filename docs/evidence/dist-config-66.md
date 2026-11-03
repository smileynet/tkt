# Standalone dist configuration — #66

`dist --version` reported cargo-dist 0.32.0. Its actual `dist migrate` command
generated `dist-workspace.toml` with `[workspace] members = ["cargo:."]` and
moved the nine dist settings verbatim out of Cargo.toml. This observed migration
supersedes the ticket's older suggestion to use the init wizard. The upstream
[configuration reference](https://axodotdev.github.io/cargo-dist/book/reference/config.html)
documents the standalone dist authority.

Local generation and sealing tooling now read that standalone authority. The
isolated generation fixture copies it and clears allow-dirty only in the temporary
copy. CI seal fixtures include it; target coverage checks remain intact. These
are tkt-owned wrappers, not modified vendored release-engine bytes.

Verification classifications:

- After migration but before wrapper updates, release-CI fixtures had three
  assertion/operational failures from the old metadata path (four passed).
- `node --test tools/release-tools.test.mjs tools/release-ci.test.mjs`: **pass**,
  38 passed, one existing Windows relative-link skip, no failures.
- `dist plan --output-format json`: **pass**, app tkt 0.4.0, all five unchanged
  targets, shell/PowerShell installers, expected tar.gz/zip archive families.
- `node tools/generate-release-workflow.mjs`: **pass**, deterministic pinned
  generation matches the existing workflow. No generated workflow diff.
- `git diff --check`: **pass**. Independent critic verified all nine equivalent
  settings, single configuration authority, wrappers and preserved stage boundary.

This is configuration/tooling work; no Rust behavior rebuild was required for
this unit. Concurrent Rust workers are isolated into separate commits and gates.
No version, tag, hosted run, release or publication was created. Plan and fixture
logs are promoted into this report, then sent to OS trash. The shared existing
target cache remains for the ongoing queue; no source/build tree was copied by
this task beyond the generator's self-cleaning small fixture.
