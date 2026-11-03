# Owner decisions — 2026-11-03

Source: the owner pasted the Decisions-for-tkt export into chat and chose all
17 recommended options. This is confirmation, not an inferred approval from
preselected controls. The dashboard records these as answered and retains them
in history. Implementation completion is separate.

| Ticket | Confirmed choice | Scheduling / effect |
|---|---|---|
| 197 | Show computed progress; close parents explicitly | Open implementation; decision gate cleared |
| 234 | Use an explicit moved marker and linked transfer | Open implementation; decision gate cleared |
| 159 | Document optional consumer CI and its limits | Open implementation; decision gate cleared |
| 202 | Design embedded versioned skill assets | Backlog preserved; implementation is not claimed complete |
| 125 | Design one explicit adapter contract first | Backlog preserved; implementation is not claimed complete |
| 157 | Keep hooks optional and advisory | Backlog preserved; implementation is not claimed complete |
| 173 | Start with bounded advisory readiness checks | Backlog preserved; implementation is not claimed complete |
| 184 | Keep narrative support deferred | Backlog preserved; implementation is not claimed complete |
| 186 | Retire the unsupported harness | Backlog preserved; implementation is not claimed complete |
| 69 | Define a focused follow-up scope | Backlog preserved; implementation is not claimed complete |
| 83 | Start with a noninteractive append operation | Backlog preserved; implementation is not claimed complete |
| 88 | Use Git closure history; mark unknown dates | Backlog preserved; implementation is not claimed complete |
| 89 | Start with advisory hooks | Backlog preserved; implementation is not claimed complete |
| 93 | Start with a narrow optional extension schema | Backlog preserved; implementation is not claimed complete |
| 148 | Use invocation identity and eligibility | Backlog preserved; implementation is not claimed complete |
| 168 | Measure demand before adding nudges | Backlog preserved; implementation is not claimed complete |
| 226 | Approve a staged implementation | Proposal remains done; implementation staged as 245 → 246 → 247 |

## Scope and scheduling

The three open implementation tickets 197, 234 and 159 now have settled owner
direction. Their acceptance criteria and validation briefs are reconciled below
the original dated triage history. Tickets 199 and 200 still depend on actual
completion of 197; this decision does not complete their prerequisite.

The 13 selected backlog items remain deliberately deferred. Settling a design
choice does not unpark a backlog item. In particular, 184 stays deferred, 168
authorizes measurement before any nudge, 125 narrows to one adapter contract,
and 186 approves a retirement that still needs implementation and evidence.

The guidance proposal approval creates real implementation stages:

1. 245: re-inventory current rules and establish verified reference homes.
2. 246: switch to conditional pointers and update the coverage matrix, after 245.
3. 247: independently audit coverage and deploy, after 246.

Stage one follows completed proposal 226. Existing operational/safety rules,
the full project code gate, local/manual hosted verification and separate release
publication authority remain intact. The proposal's old baseline must be
reconciled to the current file before extraction. No restructuring is claimed
applied by this record.

## Verification

- Pass: `tkt --agent codex validate --strict --brief` and `tkt --agent codex sync-plan --check --strict --brief`, both with zero findings.
- Pass: `node --test tools/dashboard-decisions.test.mjs tools/serve-dashboard.test.mjs`, ten tests, no failures or skips. The decision tests exercise pending-form fixtures as well as the actual answered history.
- Pass: delivery-dashboard `check-dashboard.mjs docs/status.html`, zero errors/warnings, 55 entries, 247 tickets and no open asks. The snapshot contains 208 done, six cancelled, 22 backlog, eight open and three in-progress tickets.
- Pass: proxy-disabled HTTP retrieval from the LAN dashboard matched the saved dashboard SHA-256. The existing supervised server remains active; this is not a boot-persistent service.
- Pass: `git diff --check` and scoped diff review. Changes record decisions, refine unfinished briefs, stage guidance work and verify dashboard behavior; no Rust implementation or actual AGENTS restructuring changed.
- Unverified: rendered laptop/phone appearance and interaction, because local browser use is disabled. These checks do not establish visual acceptance or complete tickets 228, 240 or 243. No Rust gate was needed for this data/docs/test-only change.
