---
id: "178"
title: "close --evidence crashes the text-mode file writer"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "tkt close <id> --check-all --evidence \"...\" completes and writes the ticket file without crashing (text mode)"
  - "A regression test covers close with --evidence in text output mode"
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

## Likely area

The text-mode ticket-file writer path invoked by `close` when `--evidence` is
present (evidence rendering into the Resolution/validation section, then the
atomic file write). JSON mode takes a different render/write path and is
unaffected. Environment: tkt 0.3.1, macOS (arm64), invoked via mise shim.

## Acceptance criteria

- [ ] `tkt close <id> --check-all --evidence "…"` completes and writes the ticket file without crashing (text mode)
- [ ] Evidence is recorded in the closed ticket (Resolution/validation section)
- [ ] A regression test covers `close` with `--evidence` in text output mode
- [ ] The JSON-mode path remains correct
