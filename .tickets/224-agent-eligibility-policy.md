---
id: "224"
title: "Let tickets declare which agents may pick them up"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "Agent eligibility parses into one typed any, any-of, or any-except policy"
  - "ready filters restricted tickets using one canonical agent identity"
  - "new, batch, edit, query, capabilities, validation, docs, and distributed skills expose the policy consistently"
tags: ["frontier", "data-model"]
---

# Let tickets declare which agents may pick them up

## What to build

Let a ticket describe which agent identities may pick it up before assignment, without
overloading work-stream tags, machine capabilities, or post-claim ownership.

The user asked to express policies such as "Codex", "Claude or Kiro", and "not
Antigravity". Persist that as one fact and parse it at the frontmatter boundary into a
sum type equivalent to:

```text
Any | AnyOf(NonEmptyAgentSet) | AnyExcept(NonEmptyAgentSet)
```

Use a validated, open-ended `AgentId` value rather than a fixed enum. Multiple names in
`AnyOf` mean any listed agent may claim the ticket; requiring several agents to work
together is a separate staffing concern. An omitted policy is `Any`, preserving every
existing ticket's behavior.

Expose an ergonomic CLI without creating two stored sources of truth: `--agents` maps to
`AnyOf`, while `--exclude-agents` maps to `AnyExcept`, and the flags are mutually
exclusive. `ready` filters by the canonical current agent identity. A mismatched or
unidentified agent receives a clear claim error and may use an explicit force override;
this is scheduling guidance, not a security boundary. `query` continues to expose the
whole corpus and reports the structured policy.

Keep the axes independent:

- `tags` remain work-stream scope selected by `tkt context`.
- `requires` remains an all-of machine-capability requirement.
- Agent eligibility controls who may pick up unclaimed work.
- `assignee` in #194 records who owns work after claim and must reuse the same `AgentId`
  and identity resolver. #194 is blocked by this ticket to prevent parallel identity
  models.

Project dashboard: `docs/status.html`.

## Context

Read `src/core/ticket.rs`, `src/config.rs`, `src/commands/ready.rs`,
`src/commands/claim.rs`, and `.tickets/194-assignee-field-and-claim.md` first. Ticket #189
made `requires` the canonical capability eligibility model; extend frontier evaluation
with a separate agent axis rather than weakening its all-of semantics. Update every
agent-facing surface listed in `.memory/agent-guidance-surfaces.md` when the CLI or
frontmatter contract changes.

## Out of scope

- Authorization or protection against an agent that deliberately changes its identity
- Multi-agent staffing requirements such as "Claude and Kiro must collaborate"
- Replacing tags, contexts, capabilities, or the future assignee field

## Acceptance criteria

- [ ] Missing `agent_eligibility` parses as `Any`; explicit any-of and any-except values round-trip without loss
- [ ] Empty agent sets, invalid agent ids, and mixed allow/exclude expressions fail once at the parse boundary with an actionable error
- [ ] `ready` includes unrestricted and matching tickets while excluding any-of misses and any-except matches
- [ ] Restricted claims with no or mismatched identity fail clearly; the documented explicit override succeeds
- [ ] `new`, `batch`, and `edit` accept mutually exclusive allow/exclude flags and serialize one canonical field
- [ ] Human and JSON `ready`/`query` output expose the policy without hiding the full corpus from `query`
- [ ] Agent eligibility composes with `requires`, tag context, dependencies, and priority without changing their existing semantics
- [ ] #194 reuses the same typed agent id and identity resolver for post-claim ownership
- [ ] Capabilities, README, init guidance, distributed skills, and changelog describe the new contract consistently
