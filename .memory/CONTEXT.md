---
type: glossary
title: "Context"
---

# Context

**Ticket**:
A single unit of work stored as `.tickets/{NN}-{slug}.md` with YAML frontmatter (id, title, status, blocked_by). Describes WHAT to build, not HOW.
_Avoid_: issue (overloaded with GitHub), task (too generic)

**Frontier**:
The set of tickets where `status: open` and all `blocked_by` are `done`. tkt works the frontier — picks the lowest-numbered available ticket (priority: high jumps the order).
_Avoid_: backlog (unordered), next (implies single item)

**Birth window**:
The period between a ticket's creation and its id being cited elsewhere. `tkt renumber` is safe only inside it — cited ids are external contracts.
_Avoid_: grace period (implies time-based)

**Frontmatter contract**:
The YAML fields tkt reads/writes: `id`, `title`, `status`, `blocked_by`, `priority`, `env`, `spec`. Other fields are preserved but not managed.
_Avoid_: schema (implies strict validation of all content)

**Surgical edit**:
A frontmatter field rewrite that preserves unknown fields, body text, and formatting of untouched areas. The core editing primitive.
_Avoid_: update (too generic), patch (implies diff format)


**GitTransaction**:
Struct in `src/transaction.rs` encapsulating the allocation transaction: fetch → scan local+remote → compute next ID → commit → push with bounded retry. Used by `new` and `batch`.
_Avoid_: allocation (too vague), push-retry (only one aspect)

**Finding**:
A single validation result from `src/findings.rs`. Has file, rule, message, severity. Produced by check functions, consumed by validate and sync-plan.
_Avoid_: error (ambiguous with exit codes), issue (overloaded)
