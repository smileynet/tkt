---
id: "160"
title: "Decide handling for 27 historical done tickets flagged missing-resolution"
status: backlog
blocked_by: []
priority: low
validation_criteria:
  - "policy decided: grandfather (suppress pre-date), backfill note, or accept as known warnings"
  - "tkt validate no longer emits noise for legacy closed tickets, OR the warnings are documented as expected"
tags: ["contract"]
---

# Decide handling for 27 historical done tickets flagged missing-resolution

## What to build

The #154 feature (validate flags done tickets with no `## Resolution` section) now surfaces 27 pre-existing warnings on early tickets (#02–#35) closed before the resolution-append behavior existed. Expected consequence of shipping the check on an existing corpus.

Decide the policy. Options (from the warn-vs-block research, `.scratch/subagent-raw/guardrail-warn-vs-block.md` — Notion ratchet pattern):
- **(a) Grandfather:** suppress `missing-resolution` for tickets closed before a cutoff date/commit. Cleanest — matches the ratchet "allow existing, block new" pattern. Needs a close-date signal (git log of the close commit, or a frontmatter marker).
- **(b) Backfill a minimal note:** add a `## Resolution (backfilled)` stub to each. Rejected-leaning — borders on fabricating closure evidence, which #154 explicitly forbids.
- **(c) Accept as documented known-warnings:** leave them, note in AGENTS.md that legacy tickets warn. Risks the Notion warn-decay trap (27 permanent warnings train users to ignore validate output).

## Context

- Do NOT fabricate resolutions (the #154 principle).
- These warnings are non-blocking (warning severity) — they only fail under `--strict`, so CI isn't broken today.
- Ratchet insight: warn should be a ramp to enforcement, not a permanent parking lot — so (a) grandfather is preferred over (c) accept.

## Acceptance criteria

- [ ] policy decided: grandfather (suppress pre-date), backfill note, or accept as documented
- [ ] `tkt validate` no longer emits noise for legacy closed tickets, OR the warnings are explicitly documented as expected
- [ ] no fabricated resolutions

## Out of scope

- Changing the #154 check itself (it's correct for new tickets)
