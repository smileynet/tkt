---
id: "85"
title: "Structured error envelopes for agent-parseable failures"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "tkt --json close 999 outputs JSON error envelope to stdout"
  - "envelope has ok, error, message, exit_code fields"
  - "without --json, stderr text output unchanged"
  - "cargo test passes"
---

# Structured error envelopes for agent-parseable failures

## Problem

tkt errors are human-readable text on stderr (`tkt: ✗ Ticket 99 not found`). Agents must regex-parse these to understand what went wrong. A structured envelope lets agents distinguish error types programmatically and decide recovery actions without brittle string matching.

## What to build

### Global `--json` flag

Promote `--json` from a ready-only flag to a global flag. When active:
- **Success**: commands that produce output emit JSON (already works for `ready --json`, `query`, `validate`)
- **Errors**: emit a JSON envelope to stdout instead of human text to stderr

```bash
tkt --json close 999
# stdout: {"ok":false,"error":"not_found","message":"no ticket with id \"999\"","exit_code":1}
# exit code: 1

tkt --json claim 01
# stdout: {"ok":true,"result":"claimed 01 auth-system (→ in_progress)"}
# exit code: 0
```

### Error envelope schema

```json
{
  "ok": false,
  "error": "<error_type>",
  "message": "<human-readable detail>",
  "exit_code": 1
}
```

### Error type vocabulary (fixed, enumerable)

| Type | Exit | Meaning |
|------|------|---------|
| `not_found` | 1 | Ticket ID doesn't exist |
| `already_done` | 1 | Ticket already closed |
| `conflict` | 1 | Push race / claim conflict |
| `gate_failed` | 1 | Quality gate blocked close (ACs, evidence, force disabled) |
| `validation` | 1 | Invalid input (bad priority, slug, status) |
| `cycle` | 1 | Dependency cycle detected |
| `io` | 2 | Filesystem or git subprocess failure |
| `parse` | 2 | Ticket file couldn't be parsed |

### Success envelope (optional — for mutation commands)

```json
{
  "ok": true,
  "result": "claimed 01 auth-system (→ in_progress)"
}
```

## Implementation

### Approach: annotated DomainError + JSON envelope on stderr

Based on research (clispec v0.3, MCP error model, Stripe/Google error taxonomies, agent recovery patterns):

**Key design decisions (research-informed):**

1. **Errors on stderr, data on stdout** — clispec consensus. JSON error envelope is the last line of stderr when `--json` active. Stdout reserved for data.
2. **Small fixed vocabulary (8 codes)** — maps to recovery strategies (retry, fix input, escalate, give up). Extensible later without breaking clients.
3. **`retryable` flag** — agents need this for automated recovery. Most tkt errors are NOT retryable; only `conflict` (push race) is.
4. **`hint` field** — what would fix it (when deterministic). Enables corrective feedback loops.
5. **Exit code = primary machine signal** — error code for routing, message for LLM reasoning. Both present.

### Error envelope (stderr, last line when --json)

```json
{"ok":false,"error":"not_found","message":"no ticket with id \"999\"","hint":"check tkt query for valid IDs","retryable":false,"exit_code":1}
```

Success envelope (stdout when --json):
```json
{"ok":true,"result":"claimed 01 auth-system (→ in_progress)"}
```

### Expanded DomainError

```rust
#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    NotFound,       // ticket/resource doesn't exist
    AlreadyDone,   // idempotency violation
    Conflict,      // push race, claim lost (RETRYABLE)
    GateFailed,    // quality gate blocked operation
    Validation,    // invalid input
    Cycle,         // dependency cycle
    Io,            // filesystem/git failure (exit 2)
    Parse,         // unparseable ticket file (exit 2)
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str { ... }
    pub fn exit_code(&self) -> i32 { ... }  // 1 for domain, 2 for operational
    pub fn retryable(&self) -> bool { ... } // only Conflict is retryable
}

#[derive(Debug)]
pub struct DomainError {
    pub code: ErrorCode,
    pub message: String,
    pub hint: Option<String>,
}
```

### Changes needed

1. `src/main.rs` — expand `DomainError` to `{ code, message, hint }`
2. `src/commands/common.rs` — update `domain_bail!` macro: `domain_bail!(NotFound, "msg")` and `domain_bail!(NotFound, "msg", hint: "hint")`
3. `src/cli.rs` — promote `--json` to global flag, emit envelope in error handler
4. Each command file — annotate `domain_bail!` calls with error code + optional hint
5. Success path — emit `{"ok": true, "result": "..."}` to stdout for mutations when `--json`

### Backward compatibility

- Without `--json`: zero change. Same stderr text, same exit codes.
- `domain_bail!("message")` without explicit code defaults to `Validation` (most common)
- Existing `ready --json` behavior preserved (it already emits JSON to stdout)

## Context

- `src/main.rs` — DomainError struct (line 21)
- `src/cli.rs` — error handler (line 424), global flags (line 9-18)
- `src/commands/common.rs` — domain_bail! macro (line 10)
- `src/commands/close.rs` — 6 domain_bail calls (not_found, already_done, gate_failed)
- `src/commands/claim.rs` — not_found, already_done, conflict
- Exit code contract: 0=success, 1=domain error, 2=operational error

## Acceptance criteria

- [ ] Global `--json` flag available on all commands
- [ ] Errors emit JSON envelope to stdout when --json active
- [ ] Error type field uses fixed vocabulary (at least 5 types)
- [ ] Human-readable message still present in envelope
- [ ] Exit codes consistent: 1=domain, 2=operational
- [ ] Without --json, stderr text output unchanged (backward compatible)
- [ ] Mutation success emits `{"ok": true, "result": "..."}` when --json
- [ ] `tkt capabilities` updated to document error types

## Out of scope

- Structured success output for all commands (query/validate already do JSON)
- Error code documentation in manpage (no manpage yet)
- Retry hints in envelope (future enhancement)
