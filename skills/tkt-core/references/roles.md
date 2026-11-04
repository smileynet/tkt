# Primary and worker coordination

Primary, implementer and critic are workflow roles chosen by the people and agent
harness. tkt does not create agents, grant delegation permission, enforce a review
role or provide `init --role`. Use authorized sessions/delegation and record work
in tickets. Give each invocation an explicit lowercase slug with global `--agent`
or `TKT_AGENT`; the flag wins. Names such as `codex-primary`, `codex-worker` and
`claude-worker` are examples, not built-in role identifiers or authenticated users.
Different concurrently acting workers should use distinct identities if their
ownership needs to be distinguishable. Reusing one identity does not make two
sessions mutually exclusive.

## Keep the fields separate

| Concept | Stored or invocation value | Meaning |
|---|---|---|
| Invocation identity | `--agent` / `TKT_AGENT` | Caller for this invocation; never a role setting in persistent config |
| Eligibility | `agents` allow or exclude policy | Which invocation identities may normally pick up an open ticket |
| Ownership | `assignee` | Invocation identity recorded by a successful identified claim; separate from eligibility |
| Workstream | `tags`, optional context | Which work is being viewed; a tag is not an agent permission |
| Machine requirements | `requires`, matched against declared `machine.capabilities` | All required machine capabilities must be present in the ready view; never agent names/roles |

`ready` applies dependency/container, machine, tag-context, eligibility and
ownership filters. Anonymous invocations see unrestricted/unowned work, with
restricted work reported separately on stderr. `query --all-context` is the
primary's whole-board read: it bypasses tag context and never narrows to the
current agent. Query exposes ownership, eligibility and pending blockers, not an
authorization verdict. Machine declarations are not hardware attestation.

Direct `claim` checks open status, remote race state, ownership and eligibility;
it does not establish dependency readiness or recheck machine capability
requirements. Check the ready view and real prerequisites before claiming.
Scheduling metadata is coordination, not an access-control boundary. For
example, ownership is not a general gate on every mutation or a permission system.

## Create a reviewed leaf for a worker

First prepare `docs/api-brief.md` with the source reference, acceptance checklist,
validation approach and boundaries. For this example the worker machine already
declares the actual `rust` capability and there is no active tag context. Replace
the illustrative IDs with the ID returned by creation.

```sh
tkt --agent codex-primary new implement-api --title "Implement API contract" --body "@docs/api-brief.md" --tags stream:api --agents codex-worker,claude-worker --requires rust --vc "Public API contract tests pass"
tkt --agent codex-worker ready
tkt --agent codex-worker claim 01
```

Creation selects eligibility, not ownership. A successful worker claim records
`assignee: codex-worker` and `in_progress`; the primary should not claim the same
leaf to signify oversight. There is no `new --assignee` or `edit --assignee`
shortcut in the current CLI. An anonymous claim of an unowned unrestricted ticket records no assignee.
Parent tickets may provide planning context and computed progress; use separate
leaf claims and close a parent explicitly only when its own criteria are met.

Before closure, the worker runs the project's verification and any required
independent critic review. The project decides how to retain that review evidence;
tkt does not automatically appoint a critic or certify its conclusions.

```sh
tkt --agent codex-worker close 01 --check-all --resolution "Implemented the reviewed API contract; independent review accepted" --evidence "Public API contract tests pass for the reviewed revision"
```

This is the close shape for one validation criterion. Replace the illustrative
resolution/evidence with observed results, and provide one truthful evidence item
per criterion. Do not close from an example's canned text. Push rejection after a
durable local commit is publication recovery, not permission to repeat a mutation.
A local-only claim is not a published coordination claim.

## Explicit handoff and force limits

Coordinate with the current worker and preserve its work before a handoff. A
primary can change eligibility with `edit --agents` or `--exclude-agents`; changing
eligibility does not clear ownership or reopen work. WIP cannot be claimed again,
even using `--force`. Do not use force to steal a live worker's ticket.

After an explicitly authorized handoff, return the ticket to open using the
supported lifecycle edit, update eligibility as needed, inspect its real blockers
and machine prerequisites, then have the new worker claim it. Reopening a normal
in-progress ticket with `edit --status open` retains its prior assignee, so an
intentional ownership override may still be needed:

```sh
tkt edit 01 --status open --agents claude-worker
tkt --agent claude-worker claim 01 --force
```

This example applies to an in-progress ticket after the old worker has stopped,
not the already-completed leaf from the previous example. Cancelled work uses
`reopen`; done work reaches completion only through `close`, never a status edit.
`--force` bypasses ownership/eligibility checks for this explicit scheduling
override. It does not make WIP open, finish blockers, supply hardware or grant
project permission. A forced claim preserves the stored eligibility policy;
update that policy explicitly rather than assuming the override rewrites it.
A successful identified claim records the new assignee.

Keep tags and machine capabilities about workstreams and execution resources.
Do not encode primary/critic/worker scheduling as `requires: [codex]`, a machine
capability named `reviewer`, or a tag-based ownership substitute. No frontier sort
override or artificial dependency is needed for these role conventions.
