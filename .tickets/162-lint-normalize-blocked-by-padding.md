---
id: "162"
title: "lint/validate: normalize blocked_by id padding and slug refs"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "tkt lint --fix pads single-digit blocked_by ids to corpus width (test: lint::normalize_blocked_by_padding)"
  - "tkt validate --fix resolves dangling-blocked-by when unique padding/slug-strip makes ref valid"
  - "tkt lint --check reports non-canonical blocked_by (no longer disagrees with validate)"
tags: ["compliance"]
---

# lint/validate: normalize blocked_by id padding and slug refs

## What to build

Discovered during a cross-project ticket audit (~/code, 20 tkt repos, 2026-08-28).
`dangling-blocked-by` from **id-format mismatch** was one of the two dominant
compliance errors across the corpus, and it recurs — the same repo re-introduced
it in newly-created tickets after an earlier hand-fix.

Two shapes observed:
1. **Padding mismatch** — `blocked_by: ["5"]` when ticket ids are zero-padded
   (`id: "05"`). Falsely blocks the dependent (ref resolves to nothing).
   Seen in gdhelper-harness (8 tickets, then 4 more from upstream) and codex_runner variants.
2. **Slug ref** — `blocked_by: ["004-spike-pi-lmstudio-integration"]` instead of
   the bare id `"004"`. Seen in local-models (8 tickets, masked until id: was added).

Today `tkt lint --check` reports "all files canonical" while these dangling-by-format
errors exist — **lint and validate disagree**. Lint should own this normalization.

Fix:
- `tkt lint` normalizes each `blocked_by` value: zero-pad numeric ids to the corpus's
  id width; strip `-slug` suffix from `NNN-slug` refs to the bare `NNN`.
- Only rewrite when the normalized ref resolves to a real ticket (deterministic, unique).
- Leave genuinely-dangling refs (no matching ticket) alone — those are real errors validate keeps.
- `tkt validate --fix` gains the same resolution so a validate pass can self-heal.

## Acceptance criteria

- [ ] `tkt lint --fix` pads single-digit blocked_by ids to corpus id width
- [ ] `tkt lint --fix` strips `-slug` suffix from `NNN-slug` blocked_by refs to bare `NNN`
- [ ] normalization only applies when the result resolves to an existing ticket (unique match)
- [ ] genuinely-dangling refs (0 matches) and ambiguous refs (>=2) are left untouched for validate to flag
- [ ] `tkt lint --check` reports non-canonical blocked_by (lint/validate no longer disagree)
- [ ] `tkt validate --fix` resolves the same dangling-by-format cases
- [ ] `lint`/`validate --fix` run twice produces no second-run diff (idempotence)
- [ ] regression test covers padding + slug-strip (lint::normalize_blocked_by_padding)

## Design (research + code-review refined, 2026-08-28)

Full findings: `.scratch/162-research/` and `.scratch/162-review/`.

### Shared resolver (single source of truth)
Three blocked_by handlers exist and must not drift: `parse_blocked_by` (read, faithful),
`normalize_blocked_by` (lint), fix.rs Tier-1 block. Put resolution in ONE new fn in
`core/ticket.rs`, called from both lint and fix:
```
pub fn resolve_blocked_by_ref(raw, &known_ids, &slug_to_id, width) -> Option<String>
// None = unresolvable -> caller leaves it for check_dangling_deps to flag
```
Plus `core::corpus_index(names) -> (HashSet<String> ids, HashMap<String,String> slug->id, usize width)`.

Reuse existing helpers — do NOT reinvent:
- `core::id_width(names)` (ticket.rs:629) — authoritative pad width, default 2, NEVER hardcode.
- `RE_NUMERIC_PREFIX` / `RE_FILENAME_ID` (ticket.rs:23-24, private) — numeric-prefix parse.
- `format!("{:0>width$}", n, width=width)` (transaction.rs:72) — exact pad idiom.
- `TicketFile::set_blocked_by(&ids)` (ticket.rs:394) — canonical write, preserves round-trip.

### Safety rules (from linter-ecosystem prior art: rustfix/clippy/rustfmt/ESLint)
1. One normalizer, two entry points — check = normalize+diff, fix = normalize+write-if-changed.
   Structurally guarantees lint/validate parity (AC: lint --check agrees with validate).
2. Unique-match guard (rustfix MachineApplicable model) — rewrite ONLY when the ref resolves to
   exactly one existing id. 0 or >=2 candidates -> leave + (fix emits advisory).
3. Reference rewrites are the documented failure class (clippy #8827). tkt has no compiler; the
   equivalent gate is: after fix, re-run validate logic and refuse to write if findings increase.
4. Idempotence: normalize(normalize(x)) == normalize(x). Test double-run produces no diff.

### Edge cases (must handle / test)
- Lint filtered-scope bug: `collect_files(dir, ids)` narrows files; build the resolver index from
  the WHOLE directory, or refs to un-linted tickets get falsely unresolved.
- Block-style values: `normalize_value` bails on `\n` (lint.rs:130) so block-style
  `blocked_by:\n  - "1"` slips past the lint rewrite path — untested gap (add coverage).
- Mixed-width corpus: pad to the matched id's width on unique match, else advisory (don't mis-pad).
- #131 regression: bare-scalar `01, 04` must never collapse to `[]` (lint.rs tests 239-260 green).
- Slug collision: build slug->id detecting dupes; skip-with-advisory, never last-wins.

### Boundaries (do NOT do — belongs to sibling tickets)
- #163 owns inserting missing `id:` — #162 assumes ids exist, only rewrites refs.
- #164 owns the ticket's own id vs filename (`id-filename-mismatch`, slug-id scheme) — #162's
  slug-strip is ONLY numeric-prefixed refs `NNN-slug` -> `NNN`, never non-numeric slug ids.

### Contract constraints
- Surgical edit via `set_field`/`set_blocked_by` only — blocked_by line changes, rest byte-identical.
  Never reformat file or touch user-owned body. Never renumber (cited ids are contracts).

### Doc surfaces (no new flags — behavior only)
commands.md (lint/validate --fix rows), AGENTS.md (CLI section), README.md (Project health);
then `bash tools/deploy-skills.sh`. No version bump (init snippets unchanged).
Gate: `cargo fmt && cargo clippy --all-targets && cargo test`, then `cargo install --path .`.
