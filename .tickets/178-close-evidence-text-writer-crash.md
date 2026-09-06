---
id: "178"
title: "close --evidence crashes the text-mode file writer"
status: in_progress
blocked_by: []
priority: high
validation_criteria:
  - "Operational errors surface the OS cause: cli.rs:526 uses {:#} so 'crash: writing <PATH>' includes the errno/source chain"
  - "tkt close <id> --check-all --evidence \"...\" completes in text mode (no 'crash: writing', ticket reaches status: done, evidence recorded)"
  - "A regression test covers text-mode close --evidence under require_validation_evidence=true (exit 0, no crash, status done, Verification section)"
---

# close --evidence crashes the text-mode file writer

## Bug

`tkt close <id> --check-all --evidence "…"` (default **text** output mode) aborts
with `tkt: ✗ crash: writing /…/NN-slug.md` and **does NOT apply the close** — the
ticket stays `in_progress`, no Resolution written, ACs unchecked. Passing
`--evidence` is the trigger; the same close **succeeds in JSON mode**.

## Repro

```bash
tkt claim NN
# … do work, check ACs …
tkt close NN --check-all \
  --evidence "criterion-1 evidence" \
  --evidence "criterion-2 evidence" \
  --resolution "what was done"
# → tkt: ✗ crash: writing …/NN-slug.md   (close NOT applied; status still in_progress)
```

- Reproduced multiple times across two tickets in a downstream project
  (operator-console), 2026-08-31 to 09-01.
- Trigger correlates with `--evidence` specifically (one or more). A `--dry-run`
  close and a close *without* `--evidence` do not crash.
- The crash is on the **file write** step, after AC/validation checks pass.

## Impact

Blocks the documented evidence-first close workflow in text mode (the default).
Users must fall back to JSON mode + hand-edit the Resolution to record evidence.

## Workaround (currently used downstream)

```bash
tkt -o json close NN --check-all --resolution "…"   # JSON mode succeeds, checks ACs
# then edit the ## Resolution section by hand to add the evidence
# re-verify: grep '^status:' .tickets/NN-*.md  → done
```

## Investigation findings (2026-09-06, code + test review — no repro yet)

A code review traced the full close→render→write path and a test review mapped existing coverage.
Full findings in `.scratch/research/178-*.md`. Key results that reshape this ticket:

**The crash is a returned I/O error, not a panic.** `tkt: ✗ crash: writing <PATH>` is produced by
exactly one site — the non-`DomainError` (anyhow) branch at `src/cli.rs:526` — wrapping the
`.with_context(|| "writing {path}")` on `std::fs::write` in `TicketFile::write()`
(`src/core/ticket.rs:408–411`). It exits **2** (operational/io). There is no `catch_unwind`/panic
hook (`main.rs:114–118`), so a Rust panic would instead print `thread 'main' panicked …` and exit
101. So the message proves the OS write call itself returned `Err` — not a slicing/format panic.

**The write path is byte-identical in text and JSON mode.** `close::run` takes no output-mode flag
and never branches on `JSON_OUTPUT` before or at the write (`close.rs:11–218`). Text vs JSON diverge
**only after** a successful write — the `!is_quiet()` reporting block (`close.rs:220–291`) and the
error-envelope shape (`cli.rs:512–527`). On code logic, the write cannot succeed under `-o json`
and fail in text mode for the **same input + same filesystem state**. The reported "JSON succeeds,
text crashes" therefore points at **differing FS state between the two runs**, not a code-path fork.

**The evidence render is byte-safe.** Rendering is inline at `close.rs:195–211`
(`format!("{}. ✓ {} — \"{}\"", i+1, criterion, ev)`); `append_resolution` (`ticket.rs:315–329`) is
pure concatenation with an idempotent skip if `## Resolution` already exists; `parse_evidence`,
`ac_section_range`, and `slug_from_filename` all slice on `str::find` char boundaries with guarded
indices. No unwrap/expect/panic site was found on the `--evidence` path.

**Could not reproduce.** The reviewer exercised unicode, quotes, embedded newlines, `%`, `{}`, named
`N=` indices, multi-criteria, over-supply, and a real bare-remote+push run against a debug build —
all exited 0 or failed *gracefully* as a `GateFailed` domain error (exit 1). Never `crash: writing`.

**The happy path is already tested and green.** `test_validation_criteria_and_evidence_flow`
(`tests/integration.rs:2059`, positional evidence, asserts `### Verification` at `:2111`) and
`test_evidence_named_mapping` (`:2128`) both close in default text mode with `--evidence` and pass.
So the current suite does **not** reproduce this bug.

### Root-cause hypotheses (ranked)

1. **Environmental `std::fs::write` failure (most likely).** Read-only/locked file, permission
   denied, a worktree/`.tickets/` path pointing somewhere non-writable, disk/quota, or an external
   process (editor / Windows AV — cf. AGENTS.md Windows notes) holding the file. The `--evidence`
   and JSON/text correlation would be **coincidental** to the specific failing run's FS state.
2. **Reporter misattribution** of a text-only post-write issue (`close.rs:220–291`). Unlikely — no
   panic site there, and the exit code (2) + message shape match an fs error, not a panic.
3. **Non-reproduced content edge case** — e.g. a pre-existing malformed `.tickets/NN-slug.md`
   frontmatter continuation line fed to `TicketFile::parse`.

### The diagnostic gap that is blocking a real fix

`cli.rs:526` prints the error with `{}` (Display), which **drops the underlying OS error / source
chain** (the errno behind `writing <PATH>`). We cannot confirm hypothesis 1 vs 3 without it. This is
the single highest-leverage change and it gates the rest of the ticket.

## Proposed fix

The prior framing ("fix the text-mode writer") is not supported by the code: the write is shared
with JSON mode and no crashing code path was found. Sequence the work diagnosis-first so we fix the
real cause rather than a guessed one.

1. **Surface the real error (do this first — it unblocks everything).** Change the surfacing at
   `cli.rs:526` from `{}` to `{:#}` (anyhow's alternate Display walks the full source chain) so
   `crash: writing <PATH>` becomes e.g. `crash: writing …/02-ev.md: Permission denied (os error
   13)`. Cheap, low-risk, and immediately distinguishes an fs/permission cause from any other. This
   improves every operational-error message, not just this one.

2. **Reproduce with the exact downstream config, then fix the confirmed cause.** The repro cites
   `require_validation_evidence = "true"` (strict) + full evidence + text mode — a combination the
   test suite never asserts as a *success*. Reproduce under that config with the errno now visible.
   - If it's an **environmental fs error** (hypothesis 1): the fix is a clearer, actionable error
     (name the path + errno + likely cause: "file may be read-only, locked, or on a read-only
     worktree"), not a writer rewrite. Consider an **atomic write** (temp-file + rename on the same
     filesystem, honoring the existing Windows delete-before-rename constraint) so a partial/failed
     write never leaves the ticket half-updated — this also addresses the "close not applied, ticket
     stuck `in_progress`" symptom regardless of the trigger.
   - If it's a **content edge case** (hypothesis 3): capture the offending `.tickets/NN-slug.md`,
     add it as a fixture, fix the parse/render.

3. **Add the missing regression test** (AC-3). Combine `require_validation_evidence = "true"` +
   full evidence + **default text mode** in a *success* assertion, and assert (a) exit 0, (b) no
   `crash: writing` in output, (c) final `status: done`, (d) `### Verification` present. This is the
   exact gap: existing strict-gate tests all assert *blocking* (exit 1); existing success tests use
   default `warn`. Test belongs in `tests/integration.rs` near the evidence cluster (~`:2059`), via
   the `run_tkt` binary harness (a suggested body is in `.scratch/research/178-tests-config.md`).

4. **Optional hardening** (only if cheap and in-scope): a text-vs-JSON parity test proving both
   modes produce byte-identical on-disk results for the same `--evidence` input (directly disproves
   the reported divergence), and an apply-twice-idempotency check on `append_resolution` (render
   prior-art recommends the "apply twice → byte-identical" invariant; verify it isn't fence-blind).

### Note on the reported text-vs-JSON divergence

Since the write is byte-identical across modes, a genuine "JSON succeeds on the same tree" result
would be new information contradicting the code reading — worth re-testing on the *same* tree (`tkt
-o json close …` vs `tkt close …`) as part of step 2. If JSON also fails there, the divergence was
an artifact of different runs.

## Acceptance criteria

- [ ] Operational errors surface the underlying cause: `cli.rs:526` uses `{:#}` (or equivalent) so `crash: writing <PATH>` includes the OS errno / source chain
- [ ] Root cause confirmed by reproduction under the downstream config (`require_validation_evidence = "true"` + full evidence + text mode), with the fix targeting the confirmed cause (clearer actionable error and/or atomic write; or a content-edge-case fix)
- [ ] `tkt close <id> --check-all --evidence "…"` in text mode completes, applies the close, and records evidence in the `### Verification` section (no `crash: writing`, ticket reaches `status: done`)
- [ ] A regression test covers text-mode `close --evidence` under the strict gate config, asserting exit 0, absence of `crash: writing`, `status: done`, and the Verification section
- [ ] The JSON-mode path remains correct (ideally a text-vs-JSON on-disk parity assertion)
- [ ] Cross-referenced with #179 (the evidence-gate error UX) — the end-to-end evidence close works in both modes, retiring the downstream "hand-edit status:done" workaround only once both land
