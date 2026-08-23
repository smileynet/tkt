---
id: "111"
title: "Explore: require or prompt for validation_criteria at tkt new time"
status: in_progress
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

**Option A** (require at creation). Reasoning:

### When is the best time to define criteria?

The quality of validation criteria degrades as you move through the lifecycle:

| Moment | Mindset | Criteria quality |
|--------|---------|-----------------|
| **Creation** | Requester (feels the pain) | Outcome-focused: "user can X", "Y no longer happens" |
| Before claim | Still understanding the problem | Same quality — equivalent to creation |
| **After claim** | Implementer (has a solution in mind) | Solution-focused: "class X exists", "test covers Y" |
| At close | Proving you're done | Writes criteria that match what was built — gaming |

### The anti-gaming argument

After claiming, the implementer is incentivized to write criteria that match what they
already built (or plan to build), not what would actually prove the work is done from
the requester's perspective. This defeats the purpose of validation criteria entirely.

For agents specifically: an agent claiming a ticket has already read it and is planning
implementation. If allowed to write criteria at that point, it'll write criteria it knows
it can satisfy.

### "I don't know the criteria yet" is usually underspecification

When someone can't write criteria at creation, it signals the ticket is underspecified —
it's a spike or research ticket. Even then, valid criteria exist:
- "Decision documented with tradeoffs"
- "Spike produces measurable result"
- "mise run verify passes"

The "I genuinely don't know" case is rarer than it seems. It usually means "I haven't
thought about what done looks like" — which is exactly the thinking `tkt new` should force.

### Implementation

`tkt new` should fail with a helpful message when `require_validation_criteria = true`
and no criteria are provided:

```
tkt: ✗ project requires validation_criteria (define what "done" looks like)
     Add: --criteria "mise run verify passes"
     Or:  --criteria "decision documented"
     Tip: write criteria from the requester's perspective, not the implementer's
```
