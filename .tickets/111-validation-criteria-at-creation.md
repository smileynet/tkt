---
id: "111"
title: "Explore: require or prompt for validation_criteria at tkt new time"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "Decision documented: enforce at creation, warn at creation, or status quo"
  - "If enforced: tkt new without criteria fails with helpful message"
---

# Explore: require or prompt for validation_criteria at tkt new time

## Problem

Projects that set `require_validation_criteria = true` in config hit friction at close
time when tickets were created without criteria. The workflow becomes:

```bash
tkt new my-ticket --title "..." --priority high   # creates without criteria ✓
# ... work happens ...
tkt close 87 --check-all --evidence "..."         # FAILS ✗
# "project config requires validation_criteria on tickets being closed"

# Workaround: manually inject criteria via sed before retrying
sed -i '' 's/^priority: high$/priority: high\nvalidation_criteria:\n  - "..."/' .tickets/87-*.md
tkt close 87 --check-all --evidence "..."         # now works ✓
```

## Evidence (gdhelper-log session, 2026-08-18)

Occurred **5 times** in one session on tickets created by both humans and Codex agents:

1. Ticket 48 — `tkt close` failed, had to inject criteria before close
2. Ticket 49 — same
3. Ticket 83 — same (wrong filename guessed first, then sed)
4. Ticket 87 — same
5. Tickets 79-86 (batch) — all 6 needed sed injection before close

Root cause: `tkt new` happily creates tickets without criteria, then `tkt close` rejects
them. The enforcement is at the wrong end of the lifecycle — the user discovers the
requirement only when they're done with the work.

Agent-created tickets are worse: Codex's `tkt new` calls never include criteria because
nothing tells it to. The agent then can't close its own tickets without `--force`.

## Options

### A. Require at creation (strict)
`tkt new` fails if `validation_criteria` is not provided when the project config requires it.
- Pro: impossible to create an uncloseable ticket
- Con: sometimes you don't know criteria upfront (exploratory work)

### B. Warn at creation, require at close (current + nudge)
`tkt new` emits a warning ("⚠ no validation_criteria — will be required for close") but
allows creation.
- Pro: flexible for quick tickets
- Con: warning is easily ignored, especially by agents

### C. Prompt/template at creation
`tkt new` auto-inserts a placeholder `validation_criteria: ["TBD"]` that must be filled
before close (close rejects "TBD" as not real criteria).
- Pro: ticket always has the field; closing forces real criteria
- Con: "TBD" clutters new tickets

### D. Allow `tkt close` to accept criteria inline
`tkt close 87 --criteria "mise run verify passes" --evidence "..."` — inject criteria
at close time without a separate sed step.
- Pro: preserves current flexibility, removes the sed workaround
- Con: late criteria may be lower quality (afterthought)

## Recommendation

**Option D** (allow criteria at close time) removes the friction without forcing upfront
planning. Optionally combine with **B** (warn at creation) to nudge agents toward
providing criteria early.
