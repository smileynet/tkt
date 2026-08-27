---
id: "156"
title: "Delta review 2026-08-27: #154 legacy-baseline gap, cross-project evidence, #152 still pending"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "each finding dispositioned: fixed, wontfix with rationale, or deferred to existing ticket"
  - "#154 spec gains a legacy-ticket baseline decision before implementation"
  - "#152 pending items re-verified, not duplicated"
tags: ["review"]
---

# Delta review 2026-08-27: #154 legacy-baseline gap, cross-project evidence, #152 still pending

## What to build

Delta review (2026-08-27, second pass) after tickets #153/#154/#155 were
filed. Verifies the new tickets' premises against source and surfaces gaps
their specs need before implementation. #152 remains the tracker for the
first-pass findings — items below are new, not duplicates.

### Review provenance

- Reviewer: opencode session, 2026-08-27 (pass 2), target HEAD `78bcd3b`
- Method: premise-verification of #153/#154/#155 claims against source; corpus scan for rollout impact; no code changes

### D1 — high: #154 has no baseline story; ~27 legacy done tickets would fire immediately

`check_resolution_quality` (src/audit.rs:12-49) flags every done ticket
lacking a `## Resolution` section. Corpus scan: 27 done tickets (02-29, 35)
predate the close protocol and have no Resolution section. The moment #154
folds this into `validate`, every run emits ~27 warnings, and
`validate --strict` (the CI gate per #154's own spec) goes red on legacy
data the team can no longer reconstruct evidence for.
Decision needed before implementing #154 — options:
1. Grandfather cutoff: only flag done tickets with id >= some threshold (or
   closed after a date), exempting the pre-protocol corpus.
2. One-time backfill sweep: append minimal retrospective Resolution stubs
   to the 27 legacy tickets (fabrication risk — #154 explicitly forbids
   `validate --fix` fabricating resolutions; a manual sweep is different but
   needs the same care).
3. Config-level exemption list in `.tickets/config.toml`.
Recommendation: option 1 (cheapest, honest); record the chosen cutoff in #154.

### D2 — low: #154 incident evidence references tickets that don't exist in this corpus

#154 cites "an agent hand-flipped #209/#210 to done" — this repo's corpus
ends at #156. The incident is cross-project (another crew repo). A fresh
agent working #154 here will search for #209/#210 and find nothing.
Fix: annotate #154's evidence line with the source project name.

### D3 — medium: #154 needs a dedup decision for audit vs validate

Once validate runs `check_resolution_quality`/`check_ac_completeness`
(src/audit.rs:12,53), `tkt audit` and `tkt validate` both report the same
findings with potentially different severity handling (audit today emits
warnings only; #154 wants errors under `--strict`). Spec should state
whether audit delegates to the shared validate path or keeps its own
behavior, and whether `tkt audit` severity changes.

### D4 — info: premises verified for #153/#154/#155

- #154: `check_resolution_quality`/`check_ac_completeness` exist at
  src/audit.rs:12 and :53 and are used only by audit
  (src/commands/audit.rs:18-19), not validate — premise accurate.
  steering/frontier-work.md exists — guidance target valid.
- #153: `tkt_bin()` (tests/integration.rs:12-20) resolves to the
  target/debug binary built by `cargo test` — never stale; premise accurate.
- #155: spike scope is sound; beads cited as prior art; no premise issues.

### D5 — info: #152 items still pending (no duplication here)

Re-verified at HEAD `78bcd3b`: fmt gate still red (F1), tickets 133-146
still TBD with no #128 refs (F3), #149 still open, F4-F7 untouched. All
tracked in #152 — disposition there, not here.

## Acceptance criteria

- [ ] D1: #154 spec gains a legacy-baseline decision (grandfather cutoff recommended) before implementation
- [ ] D2: #154 evidence line annotated with source project for the #209/#210 incident
- [ ] D3: #154 spec states audit-vs-validate dedup and severity handling
- [ ] D4/D5: no action — verification record only
- [ ] cargo fmt && cargo clippy --all-targets && cargo test pass clean (no code changes expected from this ticket)
