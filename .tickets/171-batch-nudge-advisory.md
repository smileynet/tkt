---
id: "171"
title: "new: advisory batch nudge on shared-attribute cadence (stderr + JSON hints[], TKT_ADVICE opt-out)"
status: done
blocked_by: []
priority: medium
validation_criteria:
  - "3rd consecutive new with shared tag/blocker emits stderr batch hint (test: integration::batch_nudge_trigger)"
  - "batch hint appears in -o json hints[] array (test: integration::batch_nudge_json)"
  - "TKT_ADVICE=0 suppresses the nudge (test: integration::batch_nudge_optout)"
---

# new: advisory batch nudge on shared-attribute cadence (stderr + JSON hints[], TKT_ADVICE opt-out)

## Context

Telemetry review (2026-08-30): 34 "batch-worthy" bursts (3+ `tkt new` in <1min) got no guidance — e.g. game-slicer created ~40 tickets one-by-one on Aug 6. `tkt batch` exists but agents don't reach for it.

Note: parse errors (36 `cmd=? err=parse`) were investigated and dropped — clap's "did you mean" suggestions are already enabled (`Cargo.toml:17` keeps default-features on; strsim resolves in Cargo.lock). No change needed there.

## Verified state

- `new.rs` emits no cadence hint; telemetry only records events, nothing analyzes cadence. A nudge is net-new work (`parse-batch.md` review).
- `batch` (`cli.rs:112-136`, `batch.rs:12-160`) shares flags across all created tickets and does ONE commit/push vs N — the win is largest when tickets share `--blocked-by`/`--tags`.

## Prior art (research: .scratch/research/batch-nudge.md — git's advice.* system)

A good nudge: (1) stderr only (never stdout — protects pipelines/JSON); (2) names the copy-pasteable command; (3) suggests, never auto-runs; (4) self-documenting opt-out. For automation, a prose stderr hint is INERT — agents read exit codes + stdout, so also surface it in the `-o json` envelope as a machine-legible `hints[]` array. Trigger on a shared-attribute signal (3rd consecutive `new` with same tag/blocker), not a blind counter.

## What to build

1. Session-cadence detection: track consecutive `tkt new` invocations sharing `--tags`/`--blocked-by` (key off the existing telemetry sink or a lightweight session marker).
2. On the 3rd, emit a stderr advisory naming the equivalent `tkt batch ...` command. Advisory only — never change exit code, never touch stdout, never auto-run.
3. Add a `hints[]` array to the `-o json` envelope with `code`, `suggested_command`, `disable` keys (contract change — bump capabilities, update guidance surfaces).
4. Opt-out ladder: `TKT_ADVICE=0` env kill-switch (for CI/subprocess), `-q` suppression.
5. Add a `batch` JTBD row to SKILL.md and a `batch` row to `.memory/agent-guidance-surfaces.md` coverage matrix (currently missing).

## Acceptance criteria

- [x] 3rd consecutive `new` with shared tag/blocker emits a stderr batch hint
- [x] Hint appears in `-o json` `hints[]` array with a suggested command
- [x] `TKT_ADVICE=0` and `-q` suppress the nudge
- [x] Nudge never alters stdout or exit code
- [x] SKILL.md gains a batch JTBD row; guidance-surfaces matrix gains a batch row
- [x] `cargo fmt && cargo clippy --all-targets && cargo test` clean

## Resolution (2026-08-30)

Advisory batch nudge implemented: new nudge module tracks recent 
ew calls in .git/tkt-cadence.jsonl (120s window, worktree-safe fallback); 3rd consecutive shared-tag/blocker new emits a stderr advisory (human) and a hints[] entry in the -o json success envelope (agents). Opt-out via TKT_ADVICE=0/off/false and -q. Never touches stdout or exit code. Parse-error item dropped (clap suggestions already on). Docs updated across SKILL.md/AGENTS.md/README/guidance-surfaces/capabilities.

### Verification
1. ✓ 3rd consecutive new with shared tag/blocker emits stderr batch hint (test: integration::batch_nudge_trigger) — "cargo test: 72 integration + 144 unit passed 0 failed incl batch_nudge_trigger/batch_nudge_json/batch_nudge_optout"
2. ✓ batch hint appears in -o json hints[] array (test: integration::batch_nudge_json) — "e2e installed binary 00aa832: 3rd shared-tag new emits stderr nudge (stdout clean); -o json carries hints[] with code=prefer-batch; TKT_ADVICE=0 fully suppresses"
3. ✓ TKT_ADVICE=0 suppresses the nudge (test: integration::batch_nudge_optout) — "clippy --all-targets clean; rustfmt --check clean across src+tests"
