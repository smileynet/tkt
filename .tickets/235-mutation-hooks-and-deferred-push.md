---
id: "235"
title: "Ticket mutations skip heavy commit hooks and can defer the push"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "tkt new, claim, edit and close commit only the affected ticket and existing plan files and do not run a consumer repo's heavy pre-commit gate (documented opt-in to keep hooks)"
  - "A --no-push flag and a push=false config let a ticket be created or edited locally, with the later push still collision-safe"
  - "A rejected push from a consumer pre-push hook is reported with the local commit id and a retry instruction, never silently left ahead"
tags: ["cli", "kind:enhancement", "triage:ready-for-agent", "stream:workflow"]
---

# Ticket mutations skip heavy commit hooks and can defer the push


## Intent source

Owner request on 2026-11-01 from a tagalog-helper session ("file and push tickets for grey-matter and tkt"), after a day of ticket mutations blocked by consumer hooks.

## What happened

- **tagalog-helper:** its pre-commit hook ran the full 32-stage gate, 6–8 minutes, so every `tkt new` or `tkt close` took minutes. An agent hand-wrote ticket #177 to avoid an unrequested push, because `tkt new` always commits and pushes. The repo later scoped its hook (commit `bf9726e`), but tkt still inherits whatever a consumer hook does.
- **grey-matter, 2026-11-01:**
  - `tkt new` sat for more than 10 minutes behind the opt-in `check-repo` hook.
  - A pre-push hook failure left 13 ticket commits from two concurrent sessions unpushed. tkt reported "created" but the push silently failed (`ahead 13`).
  - Interleaved `tkt new` calls from two sessions also allocated adjacent ids into each other's sequence.
- Related: #43 (mutation latency, defer-push option) and grey-matter #137, #138 and #140.

## What to build

- Ticket mutations commit with hooks skipped by default (a hook bypass scoped to that git invocation and the affected ticket/plan files), with a documented `hooks = "run"` opt-in for repos that want them.
- `--no-push` and `push = false` for local filing. The later push keeps the existing collision and renumber safety.
- A failed push names the local commit, the hook output and how to retry. The command's exit status reflects "created locally, not pushed".

## Acceptance criteria

- [ ] Fixture repo with a slow and a failing pre-commit hook: `tkt new` completes quickly and commits only affected ticket and existing plan files.
- [ ] `--no-push` and `push = false` leave the commit local; a later push renumbers safely on collision.
- [ ] Fixture with a failing pre-push hook: the exit code and message distinguish "created, not pushed".

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `cli`.

Confirmed git::commit invokes git commit -qm (hooks run), and new lacks --no-push. push.enabled=false already works; MutationContext already reports local durable commit on push failure. Extend missing behavior rather than rebuild those parts; preserve plan-file changes in mutation commits.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.
