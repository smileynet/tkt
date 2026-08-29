---
id: "167"
title: "Rebalance ticketing guidance: curb over-backlogging, promote tagging"
status: in_progress
blocked_by: []
priority: medium
validation_criteria:
  - "frontier-work.md 'Between Tickets' no longer hard-codes --status backlog for discovered work; status follows readiness (open default, backlog = deferred-out-of-cycle exception) with a positive+negative example pair"
  - "every primary ticket-creation example across all 6 guidance surfaces (per .memory/agent-guidance-surfaces.md) includes --tags in the flag list or a tag-at-creation prompt"
  - "a standalone 'when to backlog vs open' decision rule exists in frontier-work steering (top-placed, not buried mid-doc), phrased as normal imperative prose (no CRITICAL/MUST all-caps)"
  - "all 6 agent-guidance surfaces updated consistently (grep confirms no surface leads with backlog in --status enums or omits tagging in the creation workflow); deploy-skills.sh run; init snippets version-bumped if changed"
---

# Rebalance ticketing guidance: curb over-backlogging, promote tagging

## Problem

Agents exhibit two systematic ticketing misbehaviors, both traceable to what the
guidance surfaces emphasize. Confirmed quantitatively in this repo's own corpus
(167 tickets, measured 2026-08-29):

- **Over-eager backlogging** (real, moderate) — **20 of 46 live tickets (43.5%) are
  parked in `backlog`**, including one high-priority. Agents reach for `--status backlog`
  on work that is actually actionable, hiding it from the frontier.
- **Under-tagging** (strong, clear-cut) — **151 of 167 tickets (90.4%) carry no tags**;
  the first 149 tickets have zero tags (tagging is a recent partial retrofit). The
  vocabulary in use is coherent (contract, compliance, parser, review, tooling, docs, dx)
  — so the gap is adoption, not vocabulary.

Root cause is guidance emphasis, not tooling: the code review confirmed the tool has
**no counterweight** to either behavior — no validate/lint/doctor check inspects `backlog`,
and `tags` is invisible to tooling (absent from lint's `CANONICAL_ORDER`, no findings check).
Guidance is therefore the only available lever without new code (see follow-up ticket for
the optional tooling nudge).

## Research grounding (see .scratch/ticket-behavior-review/)

- **Prior art — Beads (`bd`)**, the closest analog (git-native, agent-oriented,
  dependency-aware): has **no manual backlog status in the agent happy path** — readiness
  is *computed* from the dependency graph; discovered work is captured as a
  dependency-linked issue, never parked. Validates tkt's computed-frontier design and the
  "default actionable, defer only for a named reason" direction. (research-priorart.md)
- **Backlog hygiene** — over-deferring is a named anti-pattern across Agile literature
  ("reservoir / parking lot / feature graveyard"). Fix: default to actionable, defer only
  for a named reason, treat a growing backlog as a smell. But an over-rigid readiness gate
  is itself an anti-pattern — prefer a lightweight bias, not a checklist. (research-backlog.md)
- **Tagging** — mature workflows treat a label as **mandatory intake metadata applied at
  creation**, not a deferred triage step (GitLab handbook, Langfuse "issues always have a
  label"). tkt's active-context auto-tag is arguably ahead of prior art. (research-tagging.md)
- **Agent-guidance design (drives HOW we write the fix)** — models reliably satisfy only
  ~3 concurrent constraints and under-attend mid-prompt instructions ("lost in the middle").
  Rules must be **standalone, top-placed, echoed at the end, with a positive+negative example
  pair**. Modern models follow instructions literally, so state the default AND its exception
  boundary explicitly. **All-caps CRITICAL/MUST now backfires** on current Claude models — use
  normal imperative prose. **Steering (re-injected each turn) beats skill-only** (which decays
  in long sessions) — so the primary fix belongs in `frontier-work.md` steering.
  (research-agent-guidance.md)

## What to build

Guidance surfaces that make `open` the obvious default for discovered work and prompt
tagging at creation time — without removing legitimate backlog usage. After the change,
a fresh agent reading the creation workflow should (a) reach for `open`/`blocked_by` by
default and treat `backlog` as a deliberate, named deferral, and (b) be prompted to
consider a tag when creating a ticket. The status/tag defaults should read as a single
standalone rule with one positive and one negative example, placed high in the steering
doc (not buried), in plain imperative prose.

## Context

- **Authoritative surface list (`.memory/agent-guidance-surfaces.md`) — 6 surfaces, must
  move together:**
  1. `src/commands/init.rs` — init snippets baked into the binary (AGENTS/CLAUDE/COPILOT/
     CURSOR/KIRO/WINDSURF). If changed, **bump version**.
  2. `skills/tkt/SKILL.md` — JTBD table lists backlog twice; "Backlog workflow" section has
     no counterweight; tagging is a footnote.
  3. `skills/tkt/references/commands.md` — `--status` enums list `backlog` first (default is
     `open`); `--tags` buried; canonical creation example omits it.
  4. `steering/frontier-work.md` — **HIGHEST impact + best-leverage (steering re-injects).**
     "Between Tickets" hard-codes backlog for revealed work; "Creating Tickets" conflates
     discovered=backlog; primary command omits `--tags`; "Work stream tags" paragraph siloed.
  5. `AGENTS.md` (this repo) — status line is CLEAN (keep verbatim); at most a one-line tag note.
  6. `README.md` — check status-lifecycle + tags sections for the same rebalancing.
  - **Also update (not in the 6-surface checklist but part of this problem):**
    `skills/plan-ticket-sync/SKILL.md` (Step 2 treats backlog ≈ closing dead work; Steps 4-5
    omit tags) and `skills/tkt/references/ticket-format.md` (Status Lifecycle "Enters when"
    says "Deferred or not yet prioritized" — drop "or not yet prioritized"; Principles omit tags).
- **Deploy:** `bash tools/deploy-skills.sh` (symlinks steering, per #110) after editing sources.

## Edit plan (highest leverage first)

1. **frontier-work.md (steering)** — add a top-placed standalone rule: default new/discovered
   work to `open` (or `blocked_by` for a real dependency); use `backlog` only for work
   explicitly deferred out of this cycle — with a positive+negative example pair. Rewrite
   "Between Tickets" and "Creating Tickets" accordingly. Add `[--tags STREAM]` to the primary
   command; pull "Work stream tags" into the creation workflow. Echo the rule briefly near the
   end. Plain prose, no CRITICAL/MUST.
2. **plan-ticket-sync SKILL.md** — align Step 2 to "close is the default for dead work; backlog
   only for genuinely-deferred-but-wanted"; note discovered/follow-up work is usually
   `open`/`blocked_by`. Add `--tags STREAM` to Step 4 example + a work-stream-tag MUST-have
   bullet + tags as a third ordering axis in Step 5.
3. **ticket-format.md** — tighten backlog "Enters when" (drop "or not yet prioritized"); add
   "Default to `open`, not `backlog`" + "Tag at creation" principles.
4. **commands.md** — reorder `--status` enums to lead with `open`; one-line "backlog = deferred
   out of cycle" note; add `--tags` to the canonical creation example; strengthen `--tags` desc.
5. **tkt SKILL.md** — add the "when NOT to backlog" counterweight to the Backlog workflow;
   surface tagging in the creation-quality section.
6. **init.rs snippets + README.md** — mirror the status/tag rebalancing; version-bump if init
   snippets change.
7. **AGENTS.md** — keep status line verbatim; optional one-line tag note only.

## Out of scope

- No code changes in this ticket (guidance/docs only). See follow-up ticket for the OPTIONAL
  tooling nudge (advisory on `tkt new` when a ticket lands with no tags / straight to frontier;
  or a `validate` untagged-ticket warning) — the code review identified clean hooks
  (`check_missing_tags` in findings.rs; a `new.default_status` config mirroring
  `new.default_priority`) but that is a separate, deferrable enhancement.
- Do NOT remove legitimate backlog usage — this is rebalancing, not elimination.
- Do NOT introduce a heavy Definition-of-Ready checklist (an over-rigid gate is its own
  anti-pattern per the research).

## Acceptance criteria

- [ ] frontier-work.md has a standalone, top-placed status rule (open default; backlog =
      named deferral) with a positive+negative example pair, in plain prose (no CRITICAL/MUST)
- [ ] frontier-work.md "Between Tickets" no longer hard-codes `--status backlog` for discovered work
- [ ] Every primary creation example across all 6 surfaces includes `--tags` (flag list or prompt)
- [ ] `--status` enum ordering in commands.md leads with `open`, not `backlog`
- [ ] plan-ticket-sync Step 2 makes close the default for dead work; Steps 4-5 include tags
- [ ] ticket-format.md backlog "Enters when" drops "or not yet prioritized"; tags principle added
- [ ] All 6 surfaces updated consistently (grep confirms no surface leads with backlog or omits
      tagging in the creation workflow); README + init snippets covered
- [ ] `bash tools/deploy-skills.sh` run so `~/.kiro/` reflects the edited sources; version bumped
      if init snippets changed
