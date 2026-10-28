---
id: "224"
title: "Let tickets declare which agents may pick them up"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "mise run check exits 0"
  - "cargo test agent_eligibility -- --nocapture reports all agent-eligibility tests passed"
  - "tkt capabilities reports global --agent, ticket --agents and --exclude-agents, and claim --force"
tags: ["frontier", "data-model"]
---

# Let tickets declare which agents may pick them up

## What to build

Let a ticket describe which agent identities may pick it up before assignment, without
overloading work-stream tags, machine capabilities, or post-claim ownership.

The user asked to express policies such as "Codex", "Claude or Kiro", and "not
Antigravity". Persist that as one signed list and parse it at the frontmatter boundary
into:

```rust
struct AgentId(String);
struct NonEmptyAgentSet(BTreeSet<AgentId>);

enum AgentEligibility {
    Any,
    AnyOf(NonEmptyAgentSet),
    AnyExcept(NonEmptyAgentSet),
}

enum AgentEligibilityMismatch {
    IdentityRequired,
    NotInAllowList,
    ExplicitlyExcluded,
}
```

`AgentId` is an open-ended, canonical lowercase slug built only through a validating
parser. The non-empty set rejects duplicates and serializes in deterministic order.
Multiple names in `AnyOf` mean any listed agent may claim the ticket; requiring several
agents to work together is a separate staffing concern.

Use the existing list-shaped frontmatter rather than adding a scalar expression language:

```yaml
# omitted agents field = Any
agents: ["codex"]
agents: ["claude", "kiro"]
agents: ["!antigravity"]
```

All entries must have the same sign. Positive names parse as `AnyOf`; names prefixed with
`!` parse as `AnyExcept`. A present empty list, mixed signs, duplicates, or invalid ids
fail at the parse boundary. Clearing the field restores `Any`, preserving every existing
ticket's behavior with one canonical persisted representation.

Expose an ergonomic CLI without creating two stored sources of truth: `--agents` maps to
`AnyOf`, while `--exclude-agents` maps to `AnyExcept`, and the flags are mutually
exclusive. On `edit`, an empty value removes the field.

Current identity is invocation-scoped: global `--agent <id>` overrides `TKT_AGENT`.
Do not store it in project or user configuration: Codex, Claude, Kiro, and Antigravity
can run concurrently as the same OS user on the same machine. An unidentified invocation
sees unrestricted work only; `ready` reports how many restricted tickets it hid and how
to identify the caller.

Put policy evaluation in one exhaustive method:

```rust
AgentEligibility::check(Option<&AgentId>) -> Result<(), AgentEligibilityMismatch>
```

Both `ready` and `claim` use that method. A restricted claim with no or mismatched
identity returns the typed reason; `tkt claim <id> --force` is the explicit scheduling
override. This is not a security boundary. `query` keeps its existing tag-context
behavior, does not filter by current agent, and reports the structured policy.

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

Source-backed data-model review on 2026-10-28 found:

- `ready` and `claim` are separate paths today, so a shared typed matcher is the
  single source of eligibility truth.
- Project/user config is machine- and user-scoped, but agent identity is
  invocation-scoped when several harnesses share one host.
- The narrow parser and ticket writer already support quoted lists, making one signed
  `agents` list simpler and more canonical than a new scalar expression grammar.
- `query` currently honors tag context; the new policy must not silently change that
  contract or add agent filtering there.

## Out of scope

- Authorization or protection against an agent that deliberately changes its identity
- Multi-agent staffing requirements such as "Claude and Kiro must collaborate"
- Replacing tags, contexts, capabilities, or the future assignee field

## Acceptance criteria

- [ ] Missing `agents` parses as `Any`; homogeneous positive and negative lists round-trip as deterministic `AnyOf` and `AnyExcept` sets
- [ ] Present-empty lists, duplicate ids, invalid ids, and mixed signs fail once at the parse boundary with an actionable error
- [ ] Global `--agent` overrides `TKT_AGENT`; no project or user config stores the current agent identity
- [ ] One exhaustive typed matcher returns eligible, identity-required, not-listed, or excluded outcomes and is used by both `ready` and `claim`
- [ ] `ready` includes unrestricted and matching tickets, excludes misses, and reports hidden restricted work when identity is absent
- [ ] Restricted claims fail with the typed reason; `claim --force` provides the documented explicit override
- [ ] `new`, `batch`, and `edit` accept mutually exclusive allow/exclude flags and serialize one canonical field
- [ ] Human and JSON output expose the policy; `query` remains tag-context scoped but never agent-filtered
- [ ] Agent eligibility composes with `requires`, tag context, dependencies, and priority without changing their existing semantics
- [ ] #194 reuses the same typed agent id and identity resolver for post-claim ownership
- [ ] Capabilities, README, init guidance, distributed skills, and changelog describe the new contract consistently
