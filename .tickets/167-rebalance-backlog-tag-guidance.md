---
id: "167"
title: "Rebalance ticketing guidance: curb over-backlogging, promote tagging"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "frontier-work.md 'Between Tickets' no longer hard-codes --status backlog for discovered work; status follows readiness (open default, backlog = deferred-out-of-cycle exception)"
  - "every primary ticket-creation example across all guidance surfaces includes --tags in the flag list or a tag-at-creation prompt"
  - "at least one 'when NOT to backlog' counterweight exists in the tkt skill and frontier-work steering"
  - "all agent-guidance surfaces per .memory/agent-guidance-surfaces.md updated consistently (grep confirms no surface still leads with backlog or omits tagging in creation workflow)"
---

# Rebalance ticketing guidance: curb over-backlogging, promote tagging

## Problem

Agents exhibit two systematic ticketing misbehaviors, both traceable to what the
guidance surfaces emphasize:

- **Over-eager backlogging** — agents reach for `--status backlog` on work that is
  actually actionable, hiding it from the frontier. Backlog should be reserved for
  work explicitly deferred out of the current cycle.
- **Under-tagging** — agents rarely apply `--tags` at creation, even where tags are
  the primary scoping mechanism, because tagging is buried or absent from the primary
  creation workflow.

Discovery source: multi-surface review dispatched 2026-08-29. Full per-file findings
with exact quotes and drop-in replacement text saved in
`.scratch/ticket-behavior-review/` (findings-frontier.md, findings-commands.md,
findings-ticketformat.md, findings-agents.md; criteria.md).

## What to build

Guidance surfaces that make `open` the obvious default for discovered work and prompt
tagging at creation time — without removing legitimate backlog usage. After the change,
a fresh agent reading any creation-workflow surface should (a) reach for `open`/`blocked_by`
by default and treat `backlog` as a deliberate deferral, and (b) be prompted to consider a
tag when creating a ticket.

## Context

- **Files to edit (all agent-guidance surfaces — must move together per constraint):**
  - `~/.kiro/steering/frontier-work.md` — HIGHEST impact. "Between Tickets" bullet
    hard-codes backlog for revealed work; "Creating Tickets" conflates discovered=backlog;
    primary command omits `--tags`; "Work stream tags" paragraph is siloed.
  - `skills/plan-ticket-sync/SKILL.md` (deployed to `~/.kiro/skills/`) — Step 2 treats
    backlog as interchangeable with closing dead work (its own "Superseded → close or
    rewrite" bullet is the correct model); Step 4 example + MUST-have list + Step 5
    ordering axes all omit tags.
  - `skills/tkt/SKILL.md` — Jobs-to-be-done table lists backlog twice; whole "Backlog
    workflow" section with no "when NOT to backlog"; tagging is a footnote.
  - `skills/tkt/references/ticket-format.md` — Status Lifecycle "Enters when" says
    "Deferred or not yet prioritized" (drop "or not yet prioritized"); Principles omit
    tagging; tags described passively.
  - `skills/tkt/references/commands.md` — `--status` enums list `backlog` first
    (default is `open`); `--tags` buried in flag table; canonical creation example omits it.
  - `AGENTS.md` (this repo) — status line is CLEAN (keep verbatim); at most a one-line
    tag reinforcement. Do NOT bloat.
- **Coordination constraint (AGENTS.md):** changing agent-facing guidance requires
  updating ALL surfaces together — see `.memory/agent-guidance-surfaces.md` checklist.
- **Deploy:** skills/steering deploy via `bash tools/deploy-skills.sh` (symlinks steering,
  per ticket #110). After editing repo sources, redeploy so `~/.kiro/` reflects changes.

## Edit plan (highest impact first)

1. **frontier-work.md "Between Tickets"** — rewrite so status follows readiness:
   discovered work defaults to `open` (or `blocked_by` for a real dependency); `backlog`
   only when explicitly deferred out of this cycle. Add `[--tags STREAM]` to the primary
   creation command and pull the "Work stream tags" guidance into the creation workflow
   with an imperative to tag.
2. **plan-ticket-sync SKILL.md** — align Step 2 to "close is the default for dead work,
   backlog only for genuinely-deferred-but-wanted"; note discovered/follow-up work is
   usually `open`/`blocked_by`. Add `--tags STREAM` to the Step 4 example, a work-stream-tag
   MUST-have bullet, and tags as a third ordering axis in Step 5.
3. **ticket-format.md** — tighten backlog "Enters when" (drop "or not yet prioritized");
   add a "Default to `open`, not `backlog`" principle and a "Tag at creation" principle;
   position `open` as the default.
4. **commands.md** — reorder `--status` enums to lead with `open`; add a one-line
   "backlog = deferred out of cycle" note; add `--tags` to the canonical creation example;
   strengthen the `--tags` description as the primary scoping mechanism.
5. **tkt SKILL.md** — add a "when NOT to backlog" counterweight to the Backlog workflow
   section; surface tagging in the creation-quality section, not just Key behaviors.
6. **AGENTS.md** — keep status line verbatim; optional one-line tag note only.

## Out of scope

- No code changes (this is guidance/docs only).
- No new `tkt` behavior, flags, or enforcement (that would be a separate ticket —
  e.g. a `new`-time tag prompt could be explored later but is not this ticket).
- Do NOT remove legitimate backlog usage — the goal is rebalancing, not elimination.

## Acceptance criteria

- [ ] frontier-work.md "Between Tickets" no longer hard-codes `--status backlog` for
      discovered work; status follows readiness (open default, backlog = deferral exception)
- [ ] Every primary creation example across all surfaces includes `--tags` (in the flag
      list or a tag-at-creation prompt)
- [ ] A "when NOT to backlog" counterweight exists in both the tkt skill and frontier-work
- [ ] `--status` enum ordering in commands.md leads with `open`, not `backlog`
- [ ] All surfaces in `.memory/agent-guidance-surfaces.md` updated consistently
      (grep confirms no surface leads with backlog or omits tagging in the creation workflow)
- [ ] `bash tools/deploy-skills.sh` run so `~/.kiro/` reflects the edited sources
