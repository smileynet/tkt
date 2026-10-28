---
name: tkt-orient
description: "Start-of-session pickup from the reconciled plan and tickets: read the frontier, in-progress threads, and plan ordering that tkt-reconcile left behind, flag stale WIP, and propose the next step. Use when starting a session, resuming after a break, or asking 'where were we?' / 'what's next?' / 'pick up where we left off'. Trigger: start of session, orient, resume, pick up where we left off, where were we, what's next, catch me up, continue, session start, what's on the board."
---

# Tkt Orient

Session-start bookend to `tkt-reconcile`. Read the reconciled plan + ticket state, detect anything a prior session left mid-flight, and propose a concrete next step — then hand off to the actual pick-and-work.

This skill *reads* and *orients*. It does not re-prioritize or rewrite the plan (that's `tkt-reconcile`, at session end) and it does not claim-and-execute (that's `tkt-work`) — it stops at a proposal.

Identify this invocation with `--agent <lowercase-slug>` or `TKT_AGENT` before using `ready` or `claim`. Unidentified calls see only unrestricted work; `ready` reports hidden agent-restricted tickets on stderr. `query` keeps tag context and lists all policies regardless of current agent. Agent eligibility is independent of tags and all-of machine `requires`. See `tkt-core` for allow/exclude flags and explicit claim overrides.

## Process

### 1. Read the reconciled state (internal — don't dump raw output)

```bash
tkt context                       # is a work-stream scope active? note it
tkt ready                         # the frontier: what's pickable now
tkt query --status in_progress    # claimed/mid-flight threads
```

These are instant reads — batch them into one call (with `echo ---MARKER---` separators) rather than issuing them separately.

Then read the **reconciled ordering**. The default is `docs/plan.md` — read it directly; this is what `tkt-reconcile` last re-organized and is authoritative for *intended* sequence. If `docs/plan.md` is absent, do NOT conclude "no plan" and stop: check `AGENTS.md` for the project's declared status/sequencing source — many repos deliberately use a `BACKLOG.md` or a spec overview and treat `tkt ready`/`tkt query` as authoritative. Orient from whatever source the project declares, and name the source you used. If a `HANDOFF.md` exists in the ephemeral dir (most projects won't — skip silently if absent), read it too for extra context.

### 2. Detect stale WIP

If `tkt query --status in_progress` is **empty**, there's no WIP to classify — skip straight to step 3 (still glance at `git log` once for the Heads-up freshness check: confirm the last commit was the expected `tkt-reconcile`/prior work, not a surprise).

Otherwise, an `in_progress` ticket is a claim a prior session made. Check whether the claim is real progress or an abandoned/crashed session:

```bash
git log --oneline -5             # recent commits — do any touch the in_progress ticket's files?
```

For each `in_progress` ticket, classify:
- **Live** — recent commits advance it → a resume candidate, pick it up where it stands.
- **Stale** — claimed but no commits advancing it (a seed-claim, a crash, or a context-exhausted session that never did the work). This is common — e.g. a review session that ran `tkt claim <id>` to surface the frontier but wrote no code.

For stale WIP: **flag and offer**, never auto-release. Present it as a choice — "#NN is in_progress but unstarted; resume it, or release it back to `open`?" Releasing is `tkt edit <id> --status open`, but it's the user's call (releasing another identity's claim is a judgment, and the push is the real race boundary).

**A plan-level "active thread" is not a `tkt` claim.** The plan (or a reconcile note) may say a thread was "in flight," yet nothing is `in_progress` in `tkt`. That means you're starting *fresh on that thread's next ticket*, not resuming a claim — say so, and don't hunt for a non-existent WIP.

### 3. Apply the plan-ordering rubric to find the intended next pick

Use the same rubric `tkt-reconcile` writes the plan against (do not re-derive your own order):

1. **Finish current threads first** — a live in-progress thread outranks starting a new one.
2. **Prefer earlier (lower-ID) tickets** — within/between equal threads.
3. **Blockers override position** — a blocked ticket can't be next regardless of ID.
4. **Explicit plan/user instruction overrides** — honor any recorded "do X before Y" note in the plan or handoff.
5. **Priority is contextual weight, not order** — `high`/`urgent` breaks ties among equals; it does not let a late ticket jump a live earlier thread.

The result is a *proposal*, not a claim. `tkt ready` already sorts priority-aware + lowest-ID — but **its top line is NOT automatically the pick.** When the plan's "current priorities" ordering disagrees with the frontier's first line (e.g. the frontier leads with #18 but the plan ranks #19 first), the **plan wins** (rubric item 4) — the plan encodes intent the frontier can't see. Name the divergence in your Heads-up, and state which source you followed; don't claim a false "both agree."

### 4. Present the orientation + a concrete next step

Keep it short. A distillation, not a dump:

- **Where we are** — one or two sentences: active scope (if any), how many tickets on the frontier, which thread (if any) is mid-flight.
- **Heads up** — only if there's stale WIP, a blocker cleared/appeared, or plan drift. Skip if clean.
- **Next step** — a specific proposal: "Thread X is live at #NN — resume it?" or "Frontier is clean; the rubric says #NN is next. Start?" End with a proceed question. On acceptance, hand to `tkt-work` (claim + execute the ticket).

## Verify (run up front, alongside step 1's reads)

```bash
tkt validate --brief            # structural health — surface cycles/broken refs as a Heads up
tkt sync-plan --check --brief   # plan↔ticket drift
```

Running these in the same early batch as step 1 is fine (and usually better — their results feed the orientation). Interpreting `sync-plan --check`:

- **Reports drift** (ticket/plan status disagree) → the prior reconcile is incomplete or drift crept in. Flag it and suggest running `tkt-reconcile` before picking up, rather than orienting against a stale plan.
- **Reports "no plan file" / exits non-zero because `docs/plan.md` is absent** → this is NOT drift and NOT a failure. The repo may track sequencing elsewhere (a `BACKLOG.md`, a spec overview) or rely on `tkt ready` alone (see step 1). Do **not** recommend `tkt-reconcile` — note the plan source you used and proceed. A non-zero exit here is expected, not an alarm.

Recurring cosmetic warnings (e.g. `unchecked-acs-on-done` on done-and-superseded tickets) belong in the Heads-up **once**, with a suggested one-time `tkt close --check-all` cleanup — don't re-flag them every session.

## Does NOT

- Claim and execute the ticket (that's `tkt-work` — orient proposes, tkt-work picks + works)
- Reconcile tickets or re-prioritize/rewrite the plan (that's `tkt-reconcile`, at session end — orient only reads the plan)
- Auto-release another session's `in_progress` claim (flag + offer; the user decides)
- Replace a prose handoff document — if the project keeps one, orient reads it for context but doesn't own it
