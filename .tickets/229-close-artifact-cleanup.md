---
id: "229"
title: "tkt-work close step cleans up the ticket's scratch and build artifacts"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "tkt-work's close step tells the agent to promote durable outputs, delete the ticket's scratch and build artifacts, and keep only items with a stated lifecycle scope"
  - "The Resolution names what was promoted, deleted or kept, and why anything kept is still needed"
  - "init snippets and tkt-reconcile carry the same rule without contradicting it"
tags: ["guidance"]
---

# tkt-work close step cleans up the ticket's scratch and build artifacts

## What to build

Owner's rule (2026-10-29, also grey-matter ticket 97): when closing a ticket, clean up
its scratch and build artifacts. Promote what is needed durably into tracked files and
cite those; keep an uncommitted item only while something still needs it, with a clear
lifecycle scope (which tickets, release or shared purpose, and when it ends); keep a
reasonably sized history of attempts rather than everything or nothing; delete the rest,
including build output and work directories outside the repo.

tkt ships its workflow skills standalone, so `skills/tkt-work/SKILL.md` (close step,
around "Close with evidence") states the rule in its own words instead of pointing at
grey-matter. `tkt-reconcile` checks for kept items whose scope has ended, and the `init`
snippets mention it in one line. Keep it guidance; no new command or ledger.

## Acceptance criteria

- [ ] tkt-work's close step covers promote, delete and keep-with-scope, including build artifacts outside `.scratch`
- [ ] The Resolution records what was promoted, deleted or kept, with the reason for anything kept
- [ ] tkt-reconcile flags kept items whose lifecycle scope has ended
- [ ] init snippets carry a one-line version consistent with the skill
- [ ] Repository checks (`mise run check`) pass
