# Ticket triage conventions

For triaging this CLI's unfinished tickets, use one category and one readiness tag.
Existing work-stream tags remain independent of these axes. Completed historical
tickets are audited without retrospectively inventing triage or closure evidence.

| Axis | Allowed values |
|---|---|
| `kind:` | `bug`, `enhancement` |
| `triage:` | `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix` |

A bug needs observed reproduction output before `ready-for-agent`. A source-only
suspicion remains `needs-info`, with a concrete next probe. An enhancement needs
a bounded behavioral contract, observable acceptance criteria and current paths.
`ready-for-agent` does not waive dependencies or promote deferred backlog.

`needs-info` records the specific missing reproduction or design decision in a dated
triage section. It can be investigated by an agent; design decisions stay with the
owner. `wontfix` is a disposition, not proof that work was completed. Retire only
through the lifecycle in ADR 0004 once #227 implements it; never hand-set done or
check unmet criteria to clear the board. Preserve recorded dependencies on done
tickets; add edges only for real prerequisites.

Current assessment: [2026-11-02 full-board triage](ticket-triage-2026-11-02.md).
Execution ordering: [delivery plan](../plan.md). Ticket files remain authoritative
for intent, criteria and state. This project is an app/CLI, not a game.

## Workstream graph

Each unfinished ticket has one `stream:` tag for the dashboard's six workstreams.
Domain tags remain independent; the `stream:` tag selects its primary graph lane.
Use `gate:decision` for an unresolved contract or policy choice, or `gate:evidence`
for a missing reproduction or observation. Explain the specific constraint under
`## Planning gate` in the ticket body. Generic needs-info alone is not a decision.
Remove the gate tag and explanation when resolved, then refresh the snapshot.

Planning gates do not change lifecycle state or create `blocked_by` edges. Deferred
work remains deferred, and completed dependency edges remain as history. Unblocked
in the graph means recorded prerequisites and planning gates are clear; machine
and access eligibility remain separate. Dashboard review/feedback is never a gate
assigned to the user. See [graph evidence](../evidence/dashboard-workstream-gates-240.md).
