---
type: glossary
title: "Context"
---

# Context

**Ticket**:
A single unit of work stored as `.tickets/{NN}-{slug}.md` with managed YAML frontmatter and a user-owned specification body. See the frontmatter contract below for managed fields. Describes WHAT to build, not HOW.
_Avoid_: issue (overloaded with GitHub), task (too generic)

**Frontier**:
The set of tickets where `status: open` and all `blocked_by` are `done`. tkt works the frontier — sorted by priority (urgent > high > medium > low) then lowest-numbered.
_Avoid_: backlog (unordered), next (implies single item)

**Birth window**:
The period between a ticket's creation and its id being cited elsewhere. `tkt renumber` is safe only inside it — cited ids are external contracts.
_Avoid_: grace period (implies time-based)

**Frontmatter contract**:
The YAML fields tkt manages: `id`, `title`, `status`, `blocked_by`, `priority`, `env`, `spec`, `parent`, `assignee`, `validation_criteria`, `tags`, `requires`, and signed `agents`. Unknown fields and their raw values are preserved, not necessarily interpreted.
_Avoid_: schema (implies strict validation of all content)

| Managed field | Owned meaning |
|---|---|
| `id` / `title` | Stable ticket identity and human summary |
| `status` | Stored lifecycle; only `close` may reach `done`, and cancellation stays distinct |
| `blocked_by` | Prerequisite IDs; only completed (`done`) blockers satisfy readiness |
| `priority` | Optional frontier bucket: urgent, high, medium or low; unset sorts as medium |
| `env` | Legacy single-capability compatibility field; prefer `requires` |
| `spec` | Optional durable intent/specification source |
| `parent` | Optional hierarchy parent; direct children make a ticket a frontier-excluded container |
| `assignee` | Optional owner recorded from invocation identity at claim; separate from eligibility |
| `validation_criteria` | Ordered close-time criteria, each with corresponding evidence |
| `tags` | Work-stream/context labels; label namespaces may constrain values |
| `requires` | All-of machine capabilities needed for frontier eligibility |
| `agents` | Optional homogeneous allow (`agent`) or exclude (`!agent`) invocation policy |

`TicketFile` preserves unknown frontmatter fields byte-for-byte where untouched. Cancellation metadata is body-owned (`## Cancellation`), not another managed frontmatter field.

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
Enum (`Backlog | Open | InProgress | Done | Cancelled`) in `src/core/ticket.rs`. Parsed at construction time; invalid values are rejected. `Backlog` is deferred and excluded from the frontier. `Cancelled` is terminal history, not successful completion, and cannot satisfy prerequisites. Exhaustive matching keeps every status handled.
_Avoid_: state (too generic), lifecycle (the enum is one snapshot, not the lifecycle itself)

**Consent**:
The telemetry opt-in state, checked via a priority hierarchy: `DO_NOT_TRACK=1` > `TKT_TELEMETRY` env > `CI=true` > config file > default (off). Persisted in `~/.config/tkt/consent.toml`.
_Avoid_: permission (implies access control), opt-in (describes the mechanism, not the state)

**Session ID**:
A hex string (`{timestamp_ms:012x}-{pid:04x}`) generated once per CLI invocation. Included in every telemetry event and debug trace. Enables grouping all log lines from one command run.
_Avoid_: trace ID (implies distributed tracing), correlation ID (overloaded)

**ErrorKind**:
The 8-variant enum (`NotFound | AlreadyDone | Conflict | GateFailed | Validation | Cycle | Io | Parse`) in `src/main.rs`. Fixed vocabulary for machine error classification. Exit code and retryability derived from kind. Declared in `tkt capabilities`.
_Avoid_: error code (implies numeric), error type (ambiguous with Rust types)

**Config cascade**:
Resolution order for all settings: CLI flag > env var (`TKT_{SECTION}_{KEY}`) > project config (`.tickets/config.toml`) > user config (`~/Library/Application Support/tkt/config.toml` or `XDG_CONFIG_HOME`) > built-in default. `tkt config --show` reports the winning source per key.
_Avoid_: config precedence (same thing but less discoverable)
