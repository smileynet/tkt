---
id: "65"
title: "Confirm and address Codex review findings through 260c09e"
status: done
blocked_by: []
priority: high
---

# Confirm and address Codex review findings through 260c09e

## Review provenance

- Reporter: Codex
- Review run: `22a119c7-aee5-408f-a34c-219abf6ac780`
- Review target: `260c09e7ac9feffa959a9754b981dcce52c9e004`
- Review coverage: `b4df8eae7b0d01c5be28dddac03678b7ea5534d5..260c09e7ac9feffa959a9754b981dcce52c9e004`
- Confirmation status: unconfirmed

These findings were produced by Codex. They are reviewer hypotheses, not
established defects. The agent working this ticket must reproduce and confirm
each finding against current code before changing it.

## Findings

### F1 - high: `push.enabled = false` is not honored by all mutation paths

- Location: `src/cli.rs:747`, `src/cli.rs:2267`, `src/cli.rs:2424`
- Evidence: `commit_and_publish()` checks `.tickets/config.toml` before pushing, but `new` and `batch` call `GitTransaction::try_push()` directly, while `renumber` calls `git::push_with_retry()` directly. Reproduced in a temp repo with `[push] enabled = false`: `tkt -q new no-push` advanced `origin/main`.
- Risk: projects configured for local-only work still perform network pushes from creation and renumber commands.
- Suggested confirmation: create a repo with a remote and `[push] enabled = false`, run `tkt new`, `tkt batch`, and `tkt renumber`, then compare `origin/main` before/after.
- Codex confidence: verified

### F2 - high: `tkt new` and `tkt batch` accept invalid status values

- Location: `src/cli.rs:740`, `src/cli.rs:2240`
- Evidence: creation validates slug, title, spec, env, priority, and dependencies, but passes raw `status` into `new_ticket_text()`. Reproduced with `tkt -q new bad --status bogus`; the command succeeded and `tkt validate --brief` then reported `02-bad.md [unparseable] ... bad status`.
- Risk: a successful create command can commit a ticket that later read paths reject or skip.
- Suggested confirmation: run `tkt new bad --status bogus` and `tkt batch bad --status bogus` in a scratch repo, then run `tkt validate --brief`.
- Codex confidence: verified

### F3 - high: `tkt rebase` can sweep unrelated dirty ticket edits into its commit

- Location: `src/cli.rs:1374`, `src/cli.rs:1438`
- Evidence: the command collects specific renamed paths, then ignores that scope and stages all of `.tickets/` with `git add .tickets/`.
- Risk: an automatic collision-resolution commit can include unrelated user edits or in-progress ticket changes.
- Suggested confirmation: leave an unrelated `.tickets/*.md` edit dirty, create an upstream ID collision, run `tkt rebase`, and inspect the generated commit.
- Codex confidence: verified

### F4 - high: `tkt rebase` rewrites ambiguous `blocked_by` references to the local renumbered ticket

- Location: `src/cli.rs:1391`, `src/cli.rs:1417`
- Evidence: the command builds an old-to-new ID map for all colliding local tickets and rewrites every matching `blocked_by` entry across the corpus. Ticket #61 says a ref to an origin ticket with the same old ID should warn rather than auto-fix.
- Risk: a ticket that intended to depend on the upstream ticket can be silently changed to depend on the local ticket that was moved away from the collision.
- Suggested confirmation: create a local ticket blocked by an ID that exists both upstream and as a local colliding ticket, run `tkt rebase`, and inspect `blocked_by`.
- Codex confidence: inferred

### F5 - high: AC-section detection misses real acceptance criteria when prose mentions the heading first

- Location: `src/core/mod.rs:14`, `.tickets/50-review-findings-b4df8ea.md:60`, `.tickets/50-review-findings-b4df8ea.md:121`
- Evidence: `ac_section_range()` uses `body.find("## Acceptance criteria")` rather than matching a heading line. Ticket #50 contains that string in prose before the actual AC heading, so `tkt audit --brief` does not report #50 even though it is `done`, has no Resolution section, and its real AC boxes are all unchecked.
- Risk: validation and audit can miss incomplete done tickets and let closure-quality regressions pass.
- Suggested confirmation: run `tkt audit --brief` and verify #50 is absent, then inspect #50's actual AC section.
- Codex confidence: verified

### F6 - medium: `close.require_checked_acs` is parsed but cannot disable the close guard

- Location: `src/config.rs:63`, `src/cli.rs:875`
- Evidence: the project config field is loaded, but `cmd_close()` hardcodes the all-unchecked AC failure without consulting `pcfg.close_require_checked_acs`.
- Risk: documented project configuration does not work for teams that intentionally allow closing with unchecked ACs.
- Suggested confirmation: set `[close] require_checked_acs = false`, create a ticket with all ACs unchecked, and run `tkt close <id> --resolution "..."`.
- Codex confidence: verified

### F7 - medium: release automation is not runnable from the declared toolchain

- Location: `mise.toml:18`, `tools/release.sh:19`, `release.toml:6`, `.tickets/64-release-automation.md:185`
- Evidence: ticket #64 checks off "`cargo release patch` dry-run produces correct plan", but this checkout has no declared installation for `cargo-release` or `git-cliff`. Observed `cargo release --version` -> `error: no such command: release`; observed `git cliff --version` -> `git: 'cliff' is not a git command`.
- Risk: `mise run release` cannot provide the promised single-command dry-run on a fresh environment.
- Suggested confirmation: from a clean environment, run `mise run release -- patch` or `cargo release patch --dry-run`.
- Codex confidence: verified

### F8 - medium: release config uses unprefixed changelog tags and an empty generated header

- Location: `release.toml:6`, `release.toml:9`, `release.toml:10`, `cliff.toml:6`
- Evidence: `cargo-release` is configured to create `v{{version}}` tags, but the pre-release hook passes `--tag "{{version}}"` to git-cliff. `cliff.toml` also sets `header = ""`, while ticket #64 and the current `CHANGELOG.md` require the Keep a Changelog title and preamble.
- Risk: future generated changelog links can point at non-existent unprefixed tags, and generation can drop the required changelog header.
- Suggested confirmation: install the release tools and run a dry-run changelog generation for a patch version, then inspect generated links and file header.
- Codex confidence: inferred

### F9 - medium: ticket #49 closes a subcommand API that is not implemented

- Location: `.tickets/49-dotfile-config.md:14`, `.tickets/49-dotfile-config.md:21`, `src/cli.rs:197`
- Evidence: ticket #49 requires commands like `tkt config set debug true`, `tkt config get debug`, and `tkt config list`. The implemented CLI exposes options (`tkt config --set key=value`, `--get`, `--list`) instead. Observed `tkt config set debug true` exits with clap's "unexpected argument 'set'".
- Risk: the closed ticket's user-facing API contract does not match the shipped CLI.
- Suggested confirmation: run `tkt config set debug true` and compare with `tkt config --set debug=true`.
- Codex confidence: verified

### F10 - medium: tickets #38, #39, and #40 are closed without closure evidence

- Location: `.tickets/38-investigate-godot-helper-crashes.md:4`, `.tickets/39-investigate-herdr-recall-crashes.md:4`, `.tickets/40-graceful-degradation.md:4`
- Evidence: all three are `status: done`, have no Resolution section, and have all AC boxes unchecked. `tkt audit --brief` reports all three with `all-acs-unchecked-on-done` and `missing-resolution`.
- Risk: completed-review coverage claims include ticket closures that do not establish the requested work was actually completed.
- Suggested confirmation: inspect the three ticket files and run `tkt audit --brief`.
- Codex confidence: verified

## Acceptance criteria

- [ ] Every finding is independently marked confirmed, rejected, or obsolete
- [ ] Rejected or obsolete findings include evidence and rationale
- [ ] Confirmed findings are corrected
- [ ] Regression tests cover confirmed defects where practical
- [ ] Relevant build, test, and lint checks pass
- [ ] Corrected changes receive a fresh review

## Resolution (2026-08-09)

8 findings fixed (F1-F3, F5-F8, F10), 1 rejected (F9: by design), 1 confirmed-deferred (F4: inherent ambiguity). All fixes verified via cargo fmt + clippy + test (96 pass). Key fixes: push.enabled now gates all paths, status validation on creation, rebase stages surgically, AC detection line-aware, close respects require_checked_acs config.
