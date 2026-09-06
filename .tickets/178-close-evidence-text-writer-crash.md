---
id: "178"
title: "close --evidence crashes the text-mode file writer"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "Operational errors surface the OS cause: cli.rs uses {:#} so 'crash: writing <PATH>' includes the errno/source chain (done)"
  - "Ticket writes are atomic (shared temp+rename helper) so a failed/interrupted write never leaves a torn or half-updated ticket file"
  - "fs-write failures surface as an Io environment error with an actionable hint (not 'crash'), hint printed in human output, exit code stays 2"
  - "The write-before-publish partial-state exposure in close/claim/edit is addressed or clearly surfaced (not a bare bail)"
  - "Regression tests cover text-mode close --evidence success and write-failure surfacing; updated for the new Io message shape"
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

> **Validation caveat (honest scope).** The reporter's *exact* claim — "text mode crashes, JSON mode
> succeeds on the same tree" — was **never reproduced bit-for-bit**. The fix was arrived at by
> **code-reading + reproducing the fs-error class** (a read-only file/dir yields the identical
> `crash: writing …` symptom, now surfaced with the errno). The code proves the write path is
> output-mode-independent, so the reported divergence is best explained by differing filesystem
> state between the reporter's two runs — but that specific environment was not available to confirm.
> If the symptom recurs, capture the reporter's OS, whether it was a git worktree, and the actual
> `.tickets/NN.md` that failed, and re-test with the errno now visible. The delivered fix (atomic
> write + errno-surfaced `DomainError{Io}`) addresses the fs-error class regardless.

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

### Root cause — CONFIRMED (2026-09-06, reproduced)

Hypothesis 1 is confirmed. After surfacing the errno (see Progress §1), a read-only ticket file
reproduces the **exact** reported symptom:

```
tkt: ✗ crash: writing /…/03-ro.md: Permission denied (os error 13)   # exit 2
```

JSON mode fails **identically** (exit 2, same errno) — disproving the "JSON succeeds / text crashes"
divergence, which was an artifact of differing filesystem state between the reporter's two runs. The
`--evidence` correlation was coincidental. This is an **environmental I/O failure**, not a
text-mode-writer bug and not a panic.

## Progress

**§1 — errno surfacing (DONE, committed `17d4d7c`).** `cli.rs` human message and JSON envelope now
use `{:#}` (anyhow alternate Display → full source chain). The errno was previously dropped by `{}`.
This is the highest-leverage change and it confirmed the root cause above. Verified: `mise run
check` green (fmt, clippy -D warnings, 77 tests).

**§2 — regression tests (DONE, committed `17d4d7c`).** Two tests added to `tests/integration.rs`:
`test_close_evidence_text_mode_strict_gate_writes_done` (strict gate + full evidence + text mode
success — the previously-untested downstream repro shape) and `test_close_write_failure_surfaces_os_error`
(`#[cfg(unix)]`, read-only file → asserts exit 2 + the OS errno is surfaced).

This already satisfies AC-1 and AC-4. The remaining work (below) is the substantive correctness
improvement the research surfaced.

## Additional fixes (evidence-backed — research in `.scratch/research/178b-*.md`)

The internal review surfaced a **second, more serious bug** and the error-class research shows the
current surfacing is mislabeled. These are what "completing 178 correctly" actually requires.

### A. Atomic write in a shared helper (prevents torn ticket files) — recommended

`TicketFile::write` (`ticket.rs:407–410`) is a plain `std::fs::write` — truncate-in-place, **not
atomic**. A crash / full disk / interrupt mid-write leaves a **truncated or corrupt ticket file**,
and the ticket file *is* the database. Fix with the standard pattern: write to a temp file in the
**same directory**, then rename over the target (atomic on POSIX and modern Windows).

- **Scope it as a shared `core::atomic_write(path, contents)`** helper, not just in `TicketFile::write`.
  The review found **three writers bypass `TicketFile::write`** with raw `std::fs::write`: `new.rs:132`
  (+retry), `batch.rs:119`, `lint.rs:65`. Routing all of them through one helper gives full coverage;
  `TicketFile::write` alone covers close/claim/edit/renumber/fix (6 callers).
- **Atomicity, not durability.** Ticket files are git-committed, so git is the durability backstop —
  a lost *uncommitted* write is recoverable. Atomicity (never a torn file) is the must-have; skip
  `fsync`/`F_FULLFSYNC` on the hot path (avoids latency; macOS would need `F_FULLFSYNC` for true
  durability anyway).
- **Dependency decision required.** `tempfile` is currently a **dev-dependency only** (`Cargo.toml`);
  using it in `src/` means promoting it to a runtime dep — which the "no new deps without
  justification" constraint gates. Two options: (a) justify + promote `tempfile` (cargo itself uses
  `tempfile::persist` for exactly this), or (b) hand-roll with std only (`write` temp + `fs::rename`),
  no new dep. Recommend (b) unless `tempfile`'s Windows/cleanup handling is judged worth the dep.
- **Windows caveat.** The atomic pattern renames *over* an existing file. AGENTS.md says
  `std::fs::rename` can't overwrite on Windows (delete-first, per `renumber.rs:190–194`) — **but the
  research flags this constraint as possibly stale** (modern std uses `MoveFileEx`/`ReplaceFile` and
  *can* overwrite atomically). **Verify on Windows** before choosing delete-then-rename (which
  reintroduces a non-atomic window) vs relying on atomic overwrite.

### B. Classify fs-write failures as an environment error, not a "crash" — decision required

The write failure currently surfaces as `tkt: ✗ crash: writing <path>: <errno>` (exit 2). The
error-class research (clig.dev L4, Square L3, grizzlypeak L5) is unanimous: **reserve "crash"/panic
language and the bug-class exit code for real bugs.** A read-only/locked file is an *expected
environmental condition* — it should get a plain, actionable message + hint, not "crash." This
mislabeling is precisely what seeded the downstream "tkt is broken → hand-edit status:done" folklore.

The mechanics are already in place: `domain_bail!` has a `hint:` arm (`common.rs:14–46`), `DomainError`
carries `hint`, `ErrorKind::Io` exists (but is **never produced** today), and `emit_json_error`
already serializes the hint. Convert `TicketFile::write` (and the shared helper) to
`domain_bail!(Io, "cannot write {}: {}", path, err, hint: "the file may be read-only, locked, or on a
read-only worktree — e.g. chmod +w <path>")`.

**Two gaps/decisions this forces:**
- **Human branch drops the hint.** `cli.rs:514` prints `de.message` only — it never reads `de.hint`.
  Must be fixed to print the hint (the JSON path already handles it).
- **Exit-code conflict with the contract.** clig.dev says an environment error should be exit **1**,
  but tkt's documented contract is **exit 2 = I/O** and the CLI-compat constraint forbids changing
  observable behavior. `ErrorKind::Io.exit_code()` is already **2**, so routing through
  `DomainError{Io}` **keeps exit 2** — satisfying the contract while dropping the "crash" label and
  adding a hint. Recommend keeping exit 2 (honor the contract) but removing the "crash" wording for
  the Io kind. *Do not* reclassify to exit 1 without an explicit contract decision.
- **Regression test update.** `test_close_write_failure_surfaces_os_error` asserts
  `contains("crash: writing")`; converting to `DomainError{Io}` removes the "crash:" prefix (routes
  through the DomainError branch). Update the assertion to the new message shape; exit 2 still holds.

### C. Fix the write-before-publish partial-state bug (NEW — found in review) → tracked in #180

**This is a genuine bug not in the original report.** Split out to **#180** (affects close/claim/edit
equally, not just evidence-close). In `close::run`,
`file.write()` (`close.rs:218`) runs **before** `ctx.publish()` (commit+push, `close.rs:221`), and
`MutationContext::publish` (`mutation.rs:130–146`) has **no rollback**. If the push fails (e.g.
`push_with_retry` bails after two rejections), the ticket file on disk is already `status: done` with
a Resolution appended, but the change is uncommitted/unpushed — local says done, remote does not.
`claim.rs:39–42` and `edit.rs:152–156` share the identical exposure. (The `new`/`batch` path via
`GitTransaction` *does* roll back, but only on a race-rejection, not a hard push failure.)

Options: (a) pre-flight the push-ability / write-ability *before* mutating the file (fail-fast, per
the error-class research), and/or (b) on publish failure, restore the file to its pre-write contents
(the `GitTransaction::undo_commit` pattern at `git.rs:138–169` is the model). At minimum, document
the exposure and surface a clear "written locally but not pushed — run `git push` or `tkt` will retry"
message instead of a bare bail.

### D. Cross-reference wiring + retire the folklore (AC-6) — DONE

Reference #179 in this ticket's resolution and vice versa. AGENTS.md now carries the durable lesson
(committed): *"`crash: writing …` (now shows the errno) is an fs/permission error … NOT a validation
bug; never hand-edit `status: done` to work around it."* and the Windows-rename constraint is
annotated as under verification in **#181**.

## Follow-ups (split out of this ticket)

- **#180** — write-before-publish partial-state rollback (close/claim/edit). Fix C above.
- **#181** — cross-OS verification of `core::atomic_write` (Windows `fs::rename` overwrite, temp
  cleanup, worktrees). Blocked by this ticket.
- **#179** — the evidence-gate error UX (the other half of the downstream misdiagnosis). Together
  with this ticket, retires teach-me's "hand-edit `status: done`" folklore.

### Scope recommendation

- **Must for "correct":** A (atomic write, hand-rolled std) + B (Io classification + hint, keep exit
  2) + C (at least surface the partial-state clearly; ideally pre-flight) + D (cross-ref + AGENTS).
- **Defer to follow-up tickets:** promoting `tempfile` (only if hand-roll proves insufficient),
  full power-loss durability (fsync tier), a three-class error taxonomy / new exit codes (contract
  change), the Windows `telemetry.rs:384` rename hazard, and any pre-flight TOCTOU spike.
- **Verify separately:** the AGENTS.md "Windows rename cannot overwrite" staleness question.

## Acceptance criteria

- [x] Operational errors surface the underlying cause: `cli.rs` uses `{:#}` so `crash: writing <PATH>` includes the OS errno / source chain (done, `17d4d7c`)
- [x] Root cause confirmed by reproduction under the downstream config (strict gate + full evidence + text mode): environmental `std::fs::write` failure, identical in text and JSON mode (done)
- [x] A regression test covers text-mode `close --evidence` under the strict gate (exit 0, no `crash: writing`, `status: done`, Verification section) + a write-failure test asserting the OS errno is surfaced (done, `17d4d7c`)
- [x] Ticket writes are atomic (shared `core::atomic_write` temp+rename; covers `TicketFile::write` + the `new`/`batch`/`lint` bypass writers) so a failed/interrupted write never leaves a torn or half-updated ticket file (done, `45a2227`; e2e verified zero temp leftovers)
- [x] fs-write failures surface as an environment error (Io kind) with an actionable `hint`, not "crash" language; the human branch prints the hint; exit code stays 2 (contract); the regression test is updated to the new message shape (done, `45a2227`)
- [x] The write-before-publish partial-state exposure is documented and split out to #180 (it affects close/claim/edit equally, beyond evidence-close)
- [x] The JSON-mode path remains correct — e2e verified: the Io error + hint emit identically in human and JSON modes (write path is mode-independent)
- [x] Cross-referenced with #179 — the end-to-end evidence close works in both modes; AGENTS.md gained the "crash: writing is an fs error, not a validation bug — don't hand-edit status:done" note; follow-ups split to #180/#181
