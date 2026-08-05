---
id: "38"
title: "Confirm and address review findings through b4df8ea"
status: open
blocked_by: ["39", "40", "41", "42", "43", "44", "45", "46", "47", "48"]
priority: high
---

# Confirm and address review findings through b4df8ea

## Review provenance

- Reporter: kiro-cli review session (not Codex)
- Review run: `e632f7b9-87fa-4b02-a311-f4d8ebe21c43`
- Review target: `b4df8eae7b0d01c5be28dddac03678b7ea5534d5`
- Review coverage: `208b38b82b56e1bc72ae69a13969aa793ff89845..b4df8ea` — 19 commits, plus ticket blobs for `.tickets/28`–`.tickets/37`
- Verdict: changes_requested
- Confirmation status: findings marked below as verified (reproduced this session) or reported (read from code/history). Re-confirm before changing code.

Verification run at review time (Linux, rustc/clippy 1.96):
`cargo fmt --check` → clean; `cargo clippy --all-targets` → 1 warning;
`cargo test` (unit) → 39 passed / 1 failed; `cargo test --test integration` → 29 passed.

## Findings

### F1 — high: `cargo test` is red — unit test asserts a Windows path on every platform

- Location: `src/telemetry.rs:569-576` (`telemetry::tests::project_slug_from_path`)
- Evidence: the test asserts `project_slug(Path::new("D:\\code\\game-research")) == "game-research"`. On Linux `Path::file_name` sees no separator, so the whole string is the file name. Observed failure: `left: "D:\\code\\game-research", right: "game-research"`. Because the `--bin tkt` target fails, `cargo test` stops before the integration target, so the project's stated gate (`cargo fmt && cargo clippy --all-targets && cargo test`) cannot pass on Linux.
- Pre-existing: yes — the same assertion exists at the review boundary (`git show 208b38b:src/telemetry.rs:568`). Not caused by this range.
- Risk: the verification gate quoted in AGENTS.md is unpassable on the primary dev platform, so a red suite carries no signal.
- Suggested confirmation: `cargo test --bin tkt telemetry::tests::project_slug_from_path`.
- Suggested fix: `#[cfg(windows)]` around the Windows leg, or a separator-agnostic slug helper that splits on both `/` and `\`.
- Confidence: verified

### F2 — high: closure evidence missing across the #30–#34 batch

- Location: `.tickets/30`, `31`, `32`, `33`, `34` (and `.tickets/28`)
- Evidence: all five are `status: done` with **zero** AC boxes checked (5, 5, 6, 5, 5 unchecked respectively) and no `## Resolution` section — confirmed by `tkt audit --brief` (`unchecked-acs-on-done … none checked`, `missing-resolution`). The status flips were hand-edited inside feature commits, bypassing `tkt close`: `git log -p -- .tickets/32-ready-hierarchy.md` shows `-status: open / +status: done` in `f325ea3 feat: ready hierarchy (#32) and --quiet flag (#33)`.
- Risk: `status: done` is not evidence that the specification was implemented or verified — this is exactly finding F5 of the previous review (ticket #29), recurring in the same range that claimed to address it. It also bypasses `close`'s all-unchecked guard, which exists to prevent this.
- Suggested confirmation: `tkt audit --brief`; `git log -p -- .tickets/3*.md | grep -E '^[+-]status'`.
- Suggested fix: for each ticket, check the ACs that are genuinely met with evidence, write a Resolution, and reopen or explicitly supersede the rest (F3 below lists the ones that are not met).
- Confidence: verified

### F3 — high: four test-related acceptance criteria are unmet; zero tests were added

- Location: `tests/integration.rs`
- Evidence: `#[test]` count is 29 at the boundary and 29 at HEAD; the diff for `tests/integration.rs` contains only assertion-string updates. Unmet ACs: #30 AC5 ("Integration test: close a blocker → verify unblocked tickets shown"), #33 AC5 ("Integration test: `tkt new … -q` output is a bare ID"), #34 AC5 ("Integration test with corpus containing known quality issues"), #32 AC6 ("Integration tests updated for new format" — no ready-format assertion exists).
- Risk: `close`'s unblocked line, `-q` output shapes, and the entire `audit` command ship with no regression coverage; the new output contract can silently regress.
- Suggested confirmation: `grep -c '#\[test\]' tests/integration.rs` and search for `unblocked`, `audit`, `-q` in the test file.
- Suggested fix: add the four tests, then check the ACs.
- Confidence: verified

### F4 — medium: `--check-all` checks every box in the body, not just acceptance criteria

- Location: `src/cli.rs` (`cmd_close`: `file.body = file.body.replace("- [ ]", "- [x]")`)
- Evidence: reproduced in a scratch repo. A ticket with a `## Design checklist (NOT acceptance criteria)` section containing two boxes plus one AC box: `tkt close 01 --check-all` marked all three and reported `acceptance criteria: 3/3 checked ✓`. Ticket #35 specifies "Converts all `- [ ]` to `- [x]` **in the AC section**".
- Risk: silently falsifies non-AC checklists (design checklists, task lists, review checklists) and inflates the AC count. Root cause is shared: `close`, `validate`, and `audit` all count `- [ ]`/`- [x]` body-wide with no AC-section scoping.
- Suggested confirmation: create a ticket with a non-AC checklist, run `close --check-all`, inspect the body.
- Suggested fix: scope box detection to the `## Acceptance criteria` section (one shared helper used by close, validate, and audit).
- Confidence: verified

### F5 — medium: `edit -q` still prints its confirmation

- Location: `src/cli.rs` (`cmd_edit` — the final `println!(success_msg(...))` is not gated on `is_quiet()`)
- Evidence: reproduced — `tkt edit 02 --title "Beta2" -q` prints `✓ edited 02 beta (title)`. Ticket #33's per-command table requires `tkt edit` quiet output to be nothing; `claim -q`, `close -q`, `new -q`, `ready -q` all behave as specified (verified).
- Risk: breaks the `-q` contract ("stdout: only essential data") for the one mutation command that still speaks.
- Suggested confirmation: `tkt edit <id> --title X -q` in a scratch repo.
- Suggested fix: wrap in `if !is_quiet()`. Decide the same question for `renumber` (currently always prints; not covered by #33's table).
- Confidence: verified

### F6 — medium: clippy warning present, against the project's zero-warning gate

- Location: `src/cli.rs:1444` — `projects.sort_by(|a, b| b.1.cmp(&a.1))` → `clippy::unnecessary_sort_by`
- Evidence: `cargo clippy --all-targets` → "warning: consider using `sort_by_key` … `tkt` (bin "tkt") generated 1 warning". AGENTS.md: "`cargo clippy` must produce 0 warnings".
- Pre-existing: yes — `git blame` dates the line to 2026-07-30 and it is present at `208b38b`. Possibly newly reported by a clippy upgrade.
- Suggested fix: `projects.sort_by_key(|p| std::cmp::Reverse(p.1))`.
- Confidence: verified

### F7 — low: `undo_commit_hard` no longer performs a hard reset, and leaves modifications behind

- Location: `src/git.rs:126-144`
- Evidence: the F1 fix from ticket #29 replaced `reset --hard HEAD~1` with a mixed `reset HEAD~1` plus deletion of files the commit *added* under `.tickets/`. The function name still says `_hard`. Files the undone commit *modified* (rather than added) stay modified in the worktree — e.g. inbound-ref rewrites from a failed `renumber`, or a `blocked_by` edit — where a later unrelated commit can sweep them in.
- Risk: name/behavior mismatch invites misuse; silent residue after allocation recovery.
- Suggested fix: rename to `undo_commit`, and either restore modified `.tickets/` paths explicitly (`git checkout HEAD -- <paths>`) or document the residue as intentional.
- Confidence: verified (code read; residue path not exercised)

### F8 — low: two competing quiet mechanisms

- Location: `src/cli.rs` (`static QUIET: AtomicBool` + `is_quiet()`, while `cmd_ready(json, quiet)` takes the flag as a parameter)
- Evidence: `run()` both stores `cli.quiet` into the global and threads it into `cmd_ready`; every other command reads the global.
- Risk: divergence — a future command can consult one path and miss the other.
- Suggested fix: keep one mechanism (the global, given `global = true` on the arg) and drop the parameter.
- Confidence: verified

### F9 — low: #31's colored-symbol principle is unimplemented; `NO_COLOR` AC passes vacuously

- Location: `src/cli.rs` (`success_msg`, error arm of `run()`)
- Evidence: ticket #31 specifies "**✓** (green) for success, **✗** (red) for domain errors, **⚠** (yellow) for warnings" and "Respect `NO_COLOR=1`". No ANSI sequence is emitted anywhere — `NO_COLOR=1 tkt edit …` piped through `cat -v` shows only the UTF-8 glyph. The AC "Symbols degrade gracefully when NO_COLOR=1" therefore holds trivially while the colored behavior it guards does not exist.
- Related: the error prefix changed from `tkt: <msg>` to `✗ <msg>`, dropping program identification from stderr (worth keeping `tkt:` for pipeline diagnostics); and ✓/✗/⚠/→ are non-ASCII in a project whose AGENTS.md calls out Windows portability — legacy Windows consoles can mangle them.
- Suggested fix: either implement color with `NO_COLOR`/tty detection, or amend #31 to drop the color requirement and record an ASCII-fallback decision.
- Confidence: verified

### F10 — low: small correctness and consistency nits

- `src/cli.rs` `slug_from_filename`: `split_once('-')` yields an empty slug for a filename with no dash, producing `✓ closed 01  (…)` with a blank field.
- `src/cli.rs` `cmd_audit` `stale-wip` uses filesystem mtime, which is checkout time in a fresh clone (so the check cannot fire in CI) and is reset by any `touch`. It matches #34's wording ("file mtime > 7 days"), so the spec is the thing to fix; a git commit date would be reliable.
- `src/cli.rs` `cmd_audit`'s `unchecked-acs-on-done` fires only when *nothing* is checked, while `findings.rs` uses the same rule name for *any* unchecked box. Same name, two meanings, across `audit` and `validate`.
- `src/cli.rs` `cmd_close` reloads the corpus with `if let Ok(new_corpus)` to compute the unblocked list, silently swallowing a post-write parse error.
- Confidence: verified (code read)

## Verified as correct in this range

- `src/telemetry.rs` `rotate_file`: the off-by-one fix is right — deleting `.{MAX}` first leaves every rename with a free destination, so no rename overwrites (the Windows failure mode from #29 F2 is closed). `rotate_file_respects_max_files` passes.
- `src/git.rs` `push_with_retry`: #29 F4 is now documented as a known limitation rather than silently unhandled.
- `src/telemetry.rs` `prune_oldest_sessions`: the `dead_code` allowance now explains why it is unwired (#29 F3).
- Behavior verified in a scratch repo: `ready` hierarchy with counts and indentation, `No tickets ready.`, the `→ unblocked:` line after close, `new -q` → bare id, `ready -q` → one id per line, `claim -q`/`close -q` silent, `close --resolution` writing the Resolution body.

## Acceptance criteria

- [ ] Every finding is independently marked confirmed, rejected, or obsolete
- [ ] Rejected or obsolete findings include evidence and rationale
- [ ] Confirmed findings are corrected
- [ ] Regression tests cover confirmed defects where practical
- [ ] `cargo fmt --check`, `cargo clippy --all-targets` (0 warnings), and `cargo test` all pass
- [ ] `tkt audit --brief` reports no `unchecked-acs-on-done` or `missing-resolution` for #28 and #30–#34
- [ ] Corrected changes receive a fresh review
