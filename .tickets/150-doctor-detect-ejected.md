---
id: "150"
title: "doctor should detect ejected (unparseable) tickets like validate does"
status: backlog
blocked_by: []
priority: medium
validation_criteria:
  - "doctor reports unparseable tickets that load_corpus silently skips"
  - "doctor and validate agree on ejected-ticket detection"
tags: ["parser"]
---

# doctor should detect ejected (unparseable) tickets like validate does

## What to build

`doctor` uses `core::load_corpus`, which silently skips unparseable tickets (stderr warning only). As a result `doctor` can report "All checks passed" while a ticket is ejected from the corpus. `validate` already detects this — it has its own read_dir+parse loop that emits a `Finding{rule:"unparseable", severity:"error"}` per bad file. The two commands disagree.

Make `doctor` detect ejected tickets, either by reusing validate's per-file loop or by diffing the `.md` file count against `load_corpus().len()`.

## Context

- **Relevant files:** `src/commands/doctor.rs` (Check 6 uses load_corpus), `src/commands/validate.rs` (has the unparseable-detection loop), `src/core/ticket.rs` (load_corpus)
- **Discovered during #132** — the parser leniency fixes reduce ejection frequency but don't change the silent-skip handling; detection is orthogonal
- Cross-project doctor also uses load_corpus, so its `parse_error` arm is effectively unreachable for per-ticket failures

## Acceptance criteria

- [ ] doctor reports unparseable tickets that load_corpus silently skips
- [ ] doctor and validate agree on ejected-ticket detection
- [ ] doctor does not report "all checks passed" when a ticket is ejected
- [ ] existing doctor tests pass

## Out of scope

- Changing load_corpus's return signature (that's #151)
