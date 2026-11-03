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
