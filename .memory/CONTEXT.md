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

**TicketFile**:
Raw frontmatter editor in `src/core/ticket.rs`. Owns the `Vec<(String, String)>` field map + body. Provides `set_field`, `remove_field`, `serialize`, `write`. Used for mutations; `Ticket` composes it as `ticket.file`.
_Avoid_: RawTicket (not raw — it parses), FileHandle (it's not a handle)

**Status**:
Enum (`Open | InProgress | Done`) in `src/core/ticket.rs`. Parsed at construction time — invalid values rejected before entering the corpus. Exhaustive matching ensures no status is unhandled.
_Avoid_: state (too generic), lifecycle (the enum is one snapshot, not the lifecycle itself)

**Consent**:
The telemetry opt-in state, checked via a priority hierarchy: `DO_NOT_TRACK=1` > `TKT_TELEMETRY` env > `CI=true` > config file > default (off). Persisted in `~/.config/tkt/consent.toml`.
_Avoid_: permission (implies access control), opt-in (describes the mechanism, not the state)

**Session ID**:
A hex string (`{timestamp_ms:012x}-{pid:04x}`) generated once per CLI invocation. Included in every telemetry event and debug trace. Enables grouping all log lines from one command run.
_Avoid_: trace ID (implies distributed tracing), correlation ID (overloaded)
