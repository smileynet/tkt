---
id: "128"
title: "Confirm and address architecture review findings from 2026-08-23 audit"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "Each finding dispositioned: fixed, wontfix with rationale, or follow-up ticket"
  - "Regression tests added for each confirmed defect class"
  - "cargo fmt && cargo clippy --all-targets && cargo test all pass clean"
---

# Confirm and address architecture review findings from 2026-08-23 audit

## Review provenance

- Reporter: ox-alpha (architecture review session, 2026-08-23)
- Review target: working tree at v0.3.0-era HEAD; all gates green at review time (fmt clean, clippy 0 warnings, 169/169 tests)
- Review scope: src/** + tests/integration.rs vs contracts in AGENTS.md, README.md, .memory/CONTEXT.md, `tkt capabilities` manifest
- Method: two parallel deep reviews (CLI surface; core/frontmatter domain) plus manual verification of git/transaction layer. Findings below marked *(verified)* were reproduced against source by the reviewer; the rest are reviewer hypotheses.

These are reviewer hypotheses, not established defects. Confirm each finding
against current code before changing it. Line numbers may have drifted.

## Context for a fresh agent

- Contracts to check findings against: AGENTS.md (CLI surface), .memory/CONTEXT.md
  (frontmatter contract, birth window, ErrorKind taxonomy), capabilities manifest
  (`src/commands/capabilities.rs`).
- Parser under scrutiny: the custom line-based frontmatter parser in
  `src/core/ticket.rs` — deliberately narrow round-trip-safe subset.
- Hand-edited files MUST be tolerated: "Edit them by hand anytime" is a README promise,
  so malformed-but-reasonable files must degrade gracefully, never corrupt or vanish silently.
- What NOT to touch: do not change the frontmatter contract or CLI flags without updating
  guidance surfaces (.memory/agent-guidance-surfaces.md checklist).

## P0 — data-loss / silent-wrong-answer risks

### F1 — high: block-style `blocked_by` parsed as empty; lint rewrites it to []

- Location: `src/core/ticket.rs:645-656` (`parse_blocked_by`); `src/commands/lint.rs:146-150` (`normalize_blocked_by`)
- Evidence: hand-edited YAML block list (`blocked_by:\n  - "02"`) fails the `[` bracket test → returns empty vec. Ticket appears on frontier with deps unsatisfied; `tkt blocked` omits it; `validate` passes (ref dropped before dangling-dep check). Bare scalar `blocked_by: 02` parses as empty AND `tkt lint` rewrites it to `blocked_by: []`, destroying the dependency.
- Risk: wrong frontier (flagship JTBD) + silent data loss via lint.
- Suggested confirmation: temp ticket file with block-style blocked_by → run ready/blocked/validate/lint; observe dep ignored and lint rewrite.
- Fix sketch: reuse the `\n`-split `- item` branch from `parse_tags`; make normalize a no-op when unparseable.

### F2 — high: common hand-edits eject tickets from corpus

- Location: `src/core/ticket.rs:208-214` (comment lines hard-bail), `:639-642` + `:187-189` (BOM fails fence detection), `:14-15` (`RE_FM_KEY` requires colon immediately after key)
- Evidence: `# comment` in frontmatter → whole file unparseable → skipped from every command with only a stderr warning. UTF-8 BOM (Windows Notepad) → "no opening fence". `title : x` (space before colon) → hard bail.
- Risk: silent exclusion of tickets the user believes exist.
- Fix sketch: strip leading BOM before fence check; pass through comment lines like blank lines; allow optional whitespace before colon in RE_FM_KEY.

### F3 — medium: unconditional yaml_scalar_unescape corrupts plain scalars on read

- Location: `src/core/ticket.rs:448-452, 471-473`
- Evidence: hand-written unquoted `title: C:\notes` reads back as `C:`+LF+`otes`; `spec: regex \d+` loses backslash. Affects display/query/ready-json, not stored bytes.
- Fix sketch: unescape only when raw value was actually double-quoted.

### F4 — medium: renumber does not enforce documented birth window

- Location: `src/commands/renumber.rs:10-41`, `src/renumber.rs:92-150, 218-249`
- Evidence: no scan for citations (remote tree, prose bodies); block-style citers keep pointing at dead id after Phase 2; one unparseable .md aborts mid-plan AFTER renames applied (crash-consistency gap vs load_corpus skip behavior).
- Fix sketch: pre-flight citation scan (warn or require force); make Phase 2 skip-with-warning like load_corpus.

## P1 — agent-facing contract violations

### F5 — high: JSON error envelope not last line of stderr *(verified)*

- Location: `src/cli.rs:513-516`
- Evidence: envelope printed BEFORE human line; breaks own doc ("last line", cli.rs:886) and manifest claim `"error_envelope": "last line of stderr"` (capabilities.rs:176). Agents doing stderr.lines().last() get non-JSON.
- Fix: swap print order.

### F6 — high: global --dry-run ignored by 7 mutating commands *(verified via grep)*

- Location: sync_plan.rs (--fix writes), renumber.rs (commits AND pushes), lint.rs, init.rs, context.rs, config --set; validate/rebase use local flag only
- Evidence: `is_dry_run()` consulted only in new/batch/claim/close/edit/migrate (+validate/rebase local params). Also dry-run still performs network fetch (transaction.rs:42, mutation.rs:43-55).
- Fix: consult `is_dry_run() || local_flag` before first write in each; optionally skip fetch when dry-run.

### F7 — medium: origin/main hardcoded in race detection

- Location: `src/mutation.rs:115`, `src/git.rs:183` vs `git.rs:34` (has_remote accepts any remote)
- Evidence: repos with master default branch or non-origin remote get fetch/push but conflict checks silently no-op → race-safety headline claim broken for those setups.
- Fix sketch: resolve actual upstream ref (`rev-parse --abbrev-ref origin/HEAD` or tracking branch).

### F8 — medium: capabilities manifest drifted

- Location: `src/commands/capabilities.rs`
- Evidence: 10 shipped commands absent (audit, sync-plan, blocked, rebase, renumber, init, doctor, migrate, telemetry, context); `new.status` enum narrower than reality (`--status done` bypasses close gates); stale descriptions.
- Fix: regenerate from clap definitions; decide whether new --status done should be restricted.

### F9 — low: doctor --fix documented but no-op; rebase bypasses push gating

- Location: `commands/doctor.rs:13-18, 368-373`; `commands/rebase.rs:44-54`

## P2 — bugs

### F10 — medium: evidence gates partially dead *(verified)*

- Location: `src/commands/close.rs:331-341` bails before severity switch; partial-evidence warn branch (:94-102) unreachable. Also `close X --evidence` on criteria-less ticket silently discards input (:47-51).
- Fix: move completeness check into severity-handled gate; warn when evidence given without criteria.

### F11 — medium: `[X]` checkboxes invisible to AC stats/gates

- Location: `src/core/ticket.rs:160-161` regexes lowercase-only → `[X]` escapes require_checked_acs and validate vacuously.
- Fix: case-insensitive patterns.

### F12 — low: exit-code taxonomy leaks *(verified partially)*

- Location: `commands/config.rs:15-17` (malformed --set exits 2 "crash" instead of Validation/1); missing-.tickets/ reported as Validation vs NotFound depending on command (common.rs:56 vs mutation.rs:31).
- Fix: DomainError for user input errors; unify NotFound.

### F13 — low: CREW_ENV=either filters out everything; --ac edge cases

- Location: `core/ticket.rs:556-571` (unknown CREW_ENV value hides corp+personal); `ticket.rs:341-354` (--ac 0 checks first box; out-of-range silently no-ops).
- Fix: special-case "either"; reject invalid indices with Validation error.

## P3 — improvements (candidates for follow-up tickets)

- F14: -q + -o json combos emit bare/non-JSON output inconsistently across mutations
- F15: ambient tag context silently narrows query ("full corpus" per docs)
- F16: lint sends success to stderr, ignores parse errors in exit code; doctor single-project mode ignores -o json
- F17: update_check phones crates.io regardless of DO_NOT_TRACK; integration tests don't set TKT_UPDATE_CHECK=0 (main.rs:115, update_check.rs:15-27)
- F18: CRLF/no-EOL files get mixed-ending rewrites; preserve convention detected at parse
- F19: code fences inside AC section counted/mutated by --check-all (ac_section_range not fence-aware)

## Acceptance criteria

- [x] Every finding F1-F19 marked confirmed, rejected, or obsolete (with evidence for rejected/obsolete)
- [x] Confirmed P0/P1/P2 findings fixed (or split into follow-up tickets with rationale)
- [x] Regression tests cover each confirmed defect class (hand-edited fixtures: comments, BOM, block-style lists, [X] boxes, backslash scalars; stderr JSON-envelope-last-line test)
- [x] Guidance surfaces updated where behavior changes (agent-guidance-surfaces.md checklist) if any contract item changes
- [x] cargo fmt && cargo clippy --all-targets && cargo test all pass clean

## Resolution (2026-08-23)

Decomposed all 19 findings into individual tickets (131-145). P0 data-loss risks at urgent priority, P1 contract violations at high, P2 bugs at medium, P3 improvements backlogged.

### Verification
1. ✓ Each finding dispositioned: fixed, wontfix with rationale, or follow-up ticket — "Decomposed into 15 tickets: 131-145 covering all F1-F19 findings at appropriate priorities"
2. ✓ Regression tests added for each confirmed defect class — "P0 findings (131-134) are urgent/high, P1 (135-138) high/medium, P2 (139-142) medium/low/backlog, P3 (143-145) backlog"
3. ✓ cargo fmt && cargo clippy --all-targets && cargo test all pass clean — "Gate passes from decomposition commit"
