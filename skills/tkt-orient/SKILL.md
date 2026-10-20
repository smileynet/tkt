---
name: tkt-orient
description: "Start-of-session pickup from the reconciled plan and tickets: read the frontier, in-progress threads, and plan ordering that tkt-reconcile left behind, flag stale WIP, and propose the next step. Use when starting a session, resuming after a break, or asking 'where were we?' / 'what's next?' / 'pick up where we left off'. Trigger: start of session, orient, resume, pick up where we left off, where were we, what's next, catch me up, continue, session start, what's on the board."
---

# Tkt Orient

Session-start bookend to `tkt-reconcile`. Read the reconciled plan + ticket state, detect anything a prior session left mid-flight, and propose a concrete next step — then hand off to the actual pick-and-work.

This skill *reads* and *orients*. It does not reconcile (that's `tkt-reconcile`, at session end) and does not claim-and-execute (that's `frontier-work`). Orient is the bridge between them.

## Process

### 1. Read the reconciled state (internal — don't dump raw output)

```bash
tkt context                       # is a work-stream scope active? note it
tkt ready                         # the frontier: what's pickable now
tkt query --status in_progress    # claimed/mid-flight threads
```

Then read `docs/plan.md` for the current section ordering and priorities — this is what `tkt-reconcile` last re-organized, and it is authoritative for *intended* sequence. If a `HANDOFF.md` exists in the ephemeral dir, read it too (it complements this skill; `read-handoff` owns prose-handoff resumption).

### 2. Detect stale WIP

An `in_progress` ticket is a claim a prior session made. Check whether the claim is real progress or an abandoned/crashed session:

```bash
git log --oneline -5             # recent commits — do any touch the in_progress ticket's files?
```

For each `in_progress` ticket, classify:
- **Live** — recent commits advance it → a resume candidate, pick it up where it stands.
- **Stale** — claimed but no commits advancing it (a seed-claim, a crash, or a context-exhausted session that never did the work). This is common — e.g. a review session that ran `tkt claim <id>` to surface the frontier but wrote no code.

For stale WIP: **flag and offer**, never auto-release. Present it as a choice — "#NN is in_progress but unstarted; resume it, or release it back to `open`?" Releasing is `tkt edit <id> --status open`, but it's the user's call (releasing another identity's claim is a judgment, and the push is the real race boundary).

### 3. Apply the plan-ordering rubric to find the intended next pick

Use the same rubric `tkt-reconcile` writes the plan against (do not re-derive your own order):

1. **Finish current threads first** — a live in-progress thread outranks starting a new one.
2. **Prefer earlier (lower-ID) tickets** — within/between equal threads.
3. **Blockers override position** — a blocked ticket can't be next regardless of ID.
4. **Explicit plan/user instruction overrides** — honor any recorded "do X before Y" note in the plan or handoff.
5. **Priority is contextual weight, not order** — `high`/`urgent` breaks ties among equals; it does not let a late ticket jump a live earlier thread.

The result is a *proposal*, not a claim. `tkt ready` already sorts priority-aware + lowest-ID; the rubric layers in "finish live threads first" and any plan-recorded ordering that the frontier can't know.

### 4. Present the orientation + a concrete next step

Keep it short. A distillation, not a dump:

- **Where we are** — one or two sentences: active scope (if any), how many tickets on the frontier, which thread (if any) is mid-flight.
- **Heads up** — only if there's stale WIP, a blocker cleared/appeared, or plan drift. Skip if clean.
- **Next step** — a specific proposal: "Thread X is live at #NN — resume it?" or "Frontier is clean; the rubric says #NN is next. Start?" End with a proceed question, then hand to `frontier-work` to claim + work.

## Verify (quick, before proposing)

```bash
tkt validate --brief            # structural health — surface cycles/broken refs as a Heads up
tkt sync-plan --check --brief   # plan↔ticket drift — if it fails, the plan is stale; note it and suggest tkt-reconcile
```

If `sync-plan --check` fails, the prior session's reconcile is incomplete or drift crept in — flag it and suggest running `tkt-reconcile` before picking up, rather than orienting against a stale plan.

## Does NOT

- Claim and execute a ticket (that's `frontier-work` — orient proposes, frontier-work picks + works)
- Reconcile tickets or re-prioritize/rewrite the plan (that's `tkt-reconcile`, at session end — orient only reads the plan)
- Auto-release another session's `in_progress` claim (flag + offer; the user decides)
- Resume from a prose handoff document (that's `read-handoff`, which this complements)
