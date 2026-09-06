---
id: "179"
title: "tkt close: per-criterion --evidence requirement is undiscoverable; error omits count + syntax"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "tkt close evidence error names both expected and actual counts and carries a hint with positional / N=text syntax and a copy-pasteable corrected command"
  - "docs/help make explicit that --evidence is supplied once per validation criterion (repeatable), corrected across all guidance surfaces"
tags: ["cli"]
---

# tkt close: per-criterion --evidence requirement is undiscoverable; error omits count + syntax

## Problem

`tkt close` requires **one `--evidence` per validation criterion**, but nothing surfaces that
contract until you fail, and the failure message doesn't tell you (a) how many criteria need
evidence or (b) the syntax to satisfy them. A user with a valid, ready-to-close ticket gets a
terse rejection and no path forward — which reads as "tkt close is broken" rather than "supply
N evidence values."

Related but distinct from **#178** (`close --evidence` crashes the text-mode writer): #178 is a
crash on the *write*; this is the *validation error's quality/discoverability* on the way in.
Both together explain a downstream misdiagnosis (see Impact).

## Repro (tkt 0.3.1, macOS, dry-run — no mutation)

A ticket with 3 `validation_criteria`, closed with a single `--evidence`:

```bash
tkt close 316 --check-all --resolution "x" --evidence "x" --dry-run
# → tkt: ✗ --evidence: criteria 2, 3 have no evidence (provide positional or named evidence for each)
```

Supplying one `--evidence` per criterion works:

```bash
tkt close 316 --check-all --resolution "x" --evidence e1 --evidence e2 --evidence e3 --dry-run
# → Would close 316 … Verification: 3 criteria with evidence
```

The requirement (one evidence per criterion) is correct policy; the **problem is it's
undiscoverable**: the error names *which* criteria lack evidence but not the total count, doesn't
show the repeatable/positional/`N=text` form, and the requirement isn't stated in `--help` or
pre-flight.

## Impact — causes downstream misattribution

A downstream project (teach-me) hit this (compounded by #178's crash) and wrote into its AGENTS.md:
"`tkt close` has a config validation bug that rejects valid tickets — edit frontmatter
`status: done` directly instead." That guidance **bypasses the close gates entirely** (AC checks,
resolution, evidence, atomic push) — the exact opposite of what the evidence-first workflow wants.
A clearer error + help text would have prevented the workaround becoming project policy.

## Proposed fix

Research (CLI error-UX guidelines + prior art) and a code/docs review sharpened the fix. See the
**Research appendix** below for sources and the **Code map** for exact sites. The requirement and
parsing are already correct — the gap is purely message quality and doc coverage.

### 1. Rewrite the evidence gate error (`src/commands/close.rs:362–370`)

Adopt the what/why/how shape all three consulted guidelines converge on (clig.dev, Tidyverse,
"12 Rules of Great CLI UX"): a "must" problem statement naming **both** the expected and actual
count, then the exact syntax, then a **copy-pasteable corrected command last** (the eye lands on
the tail). Everything needed is already in scope at the error site — `criteria_count`, `unfilled`,
and `unfilled.len()` — so this is a format-string + `hint:` change with no new plumbing.

Current:

```
--evidence: criteria 2, 3 have no evidence (provide positional or named evidence for each)
```

Proposed (message + hint; `domain_bail!(GateFailed, "…", hint: "…")` — the hint also renders in the
`-o json` error envelope, so JSON consumers get the fix too):

```
✗ --evidence: 2 of 3 validation criteria still need evidence (missing: 2, 3).
  hint: supply one --evidence per criterion (positional in order, or N=text to target one):
        tkt close 316 --check-all --resolution "…" \
          --evidence "<criterion 1>" --evidence "<criterion 2>" --evidence "<criterion 3>"
```

- Name **expected AND actual**: "2 of 3 … need evidence" (not just which indices).
- Show the **exact repeatable syntax** and the `N=text` alternative.
- End with a **corrected, runnable command** using `<…>` placeholders for the missing slots.
- Keep it on stderr (already the case for `domain_bail!`). Under-supply is misuse → the exit
  code should be in the misuse class, matching tkt's existing convention (1 = domain failure).
- Add a **singular/plural guard** ("1 criterion" vs "N criteria"); tkt is English-only, so no i18n.
- **Optional stretch:** echo the criterion *titles*, not just indices ("missing: 2 (\"…\"), 3
  (\"…\")"). Requires passing `&criteria` instead of `criteria.len()` into `parse_evidence`
  (`close.rs:43`) — small local signature change, data is already at the call site. Titles make
  the fix self-documenting but risk long lines; gate behind whether criterion text is short.

### 2. Expand `--evidence` help to state the one-per-criterion contract (`src/cli.rs:167`)

Current help — "Evidence for validation criteria (repeatable, positional or N=text for named)" —
names the mechanics but not the **one-per-criterion requirement**. clap does **not** auto-annotate
repeatability in `--help`; it must be written into the help text (the `<VALUE>...` usage suffix is
the only automatic multiplicity signal). Prior art (git `-m`, kubectl `--from-literal`, gh `-f`)
always states repeat semantics in prose. Proposed:

```
/// Evidence of validation — supply one per validation criterion. Repeatable:
/// positional items fill criteria in order, or use N=text to target criterion N.
```

### 3. Fix the doc surfaces that under-represent the contract (the actual trap)

The single-string example `--evidence "All tests pass"` appears across every guidance surface and
misleads on multi-criterion tickets. This repo sets `require_validation_evidence = "true"` (a
**hard gate**, not warn — `.tickets/config.toml:5–9`), so the omission reliably blocks agents.
Update to show the repeatable, one-per-criterion form (and note it where the `--evidence`
mechanics are relevant):

- `README.md:112` (example) and `:187` (prose)
- `AGENTS.md:53` (synopsis — mark `--evidence` repeatable like sibling flags), `:89`, `:97`
- `steering/frontier-work.md:46–49` ("Evidence must be provided" → "one `--evidence` per criterion")
- `src/commands/init.rs:26,57,88,98,118` (scaffolds copied into consumer projects — same gap,
  propagated to every `tkt init`)

Per the AGENTS.md constraint, changing agent-facing `--evidence` guidance means updating **all**
guidance surfaces together (init snippets, steering, AGENTS.md, README) — see
`.memory/agent-guidance-surfaces.md`.

### 4. Cross-reference #178 (AC-3)

#178 is a **separate, later-stage** bug: `close --evidence` crashes the text-mode file writer on
the *write* step, downstream of the `evidence_map` computed at `close.rs:41–45` — independent of
`parse_evidence`. Fixing #179's message does not touch the writer and vice versa, but both must
land for the downstream misdiagnosis (teach-me's "hand-edit `status: done`" folklore) to be fully
retired. Note the relationship in both tickets' resolutions.

### Deliberately out of scope

- The `N=text` **indexed** syntax has **no first-tier prior art** (git/kubectl/gh/clap all use
  repeat-order or in-value brackets like gh's `key[]=value`). It already exists in tkt, so this
  ticket keeps and documents it rather than redesigning it — but a future ticket could reconsider
  whether the indexed form earns its novelty.
- The optional pre-flight hint on `tkt show`/`tkt ready` (surfacing "N criteria need N evidence"
  before close) is a nice-to-have, not required by the ACs. Track separately if wanted.

## Acceptance criteria

- [ ] `tkt close` evidence error names **both** counts (e.g. "2 of 3 validation criteria still need evidence") — not just which indices — with a correct singular/plural form
- [ ] The error carries a `hint:` showing the repeatable/positional syntax AND the `N=text` form, ending with a copy-pasteable corrected command using `<…>` placeholders (renders in `-o json` too)
- [ ] `tkt close --help` (`--evidence` doc at `src/cli.rs:167`) states `--evidence` is supplied **once per validation criterion** (repeatable; positional-in-order or `N=text`)
- [ ] The single-string `--evidence "…"` folklore is corrected across all guidance surfaces (README, AGENTS.md, steering/frontier-work.md, init.rs scaffolds) per the guidance-surfaces checklist
- [ ] Cross-referenced with #178 (the text-mode writer crash): the end-to-end close-with-evidence path is verified working (not just the error message in isolation), and both resolutions reference each other so the downstream "hand-edit status:done" folklore is retired only once both land

## Notes

- Root of a real downstream cost: teach-me's AGENTS.md close-instruction folklore traces to this
  (+ #178). Fixing both lets that project drop its hand-edit-status workaround.

## Code map (verified 2026-09-06)

| What | Location |
|------|----------|
| Evidence gate error (`have no evidence`) | `src/commands/close.rs:362–370` (format string `:364`) |
| `parse_evidence(evidence, criteria_count)` | `src/commands/close.rs:308` |
| Named `N=text` out-of-range / duplicate bails | `close.rs:317–333` |
| Positional overflow bail | `close.rs:341–347` |
| `unfilled` computation (1-based) | `close.rs:353–359` |
| Call site — passes `criteria.len()` | `close.rs:43` (would pass `&criteria` for title echo) |
| CLI `--evidence` arg + help | `src/cli.rs:167–169` (`num_args=1, action=Append`) |

At the error site, `criteria_count`, `unfilled`, and `unfilled.len()` are all in scope — the count
and syntax are simply not interpolated today. `domain_bail!` supports a `hint:` form that renders in
the `-o json` error envelope, so the corrected-command hint reaches JSON consumers too.

## Research appendix

Full findings in `.scratch/research/179-*.md`.

**Error-UX best practice** (clig.dev [L4], Tidyverse error style guide [L4], "12 Rules of Great CLI
UX" [L5]): every error states what/why/how; name **both** expected and actual counts ("must have
length 1, not 2"); show the exact syntax; end with a copy-pasteable corrected command (fix goes
**last** — that's where the eye lands); send to stderr; treat under-supply as misuse. Use `<value>`
placeholders; **show, don't auto-run** (right default for an agent-driven CLI). Map into
`DomainError { message, hint }` as message = the count mismatch, hint = the corrected command. Add a
singular/plural guard.

**Prior art for repeatable flags** (git `-m`, kubectl `--from-literal`, gh `-f`, clap
`ArgAction::Append`): the universal convention is the **repeated flag**, documented via (a) a
metavar encoding item shape (`<key=value>`) and (b) prose stating repeat semantics ("may be given
multiple times"). **No first-tier tool uses an `N=value` indexed form** — tkt's `N=text` is novel
(closest analog is gh's `key[]=value` bracket syntax or plain repeat-order). clap does not
auto-annotate repeatability in `--help`; write it into the help text. If occurrences combine, name
the separator explicitly (git's `-m` paragraph-join surprises users).
