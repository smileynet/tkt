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

### Approach: error type annotation on DomainError

Currently `DomainError(String)` is untyped. Change to `DomainError { code: ErrorCode, message: String }`:

```rust
#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    NotFound,
    AlreadyDone,
    Conflict,
    GateFailed,
    Validation,
    Cycle,
    Io,
    Parse,
}

#[derive(Debug)]
pub struct DomainError {
    pub code: ErrorCode,
    pub message: String,
}
```

Then in the error handler (cli.rs line 424):
- If `--json` flag is active: serialize envelope to stdout
- Else: print human text to stderr (current behavior)

### Changes needed

1. `src/main.rs` — expand `DomainError` to carry `ErrorCode`
2. `src/commands/common.rs` — update `domain_bail!` macro to accept error code
3. `src/cli.rs` — promote `--json` to global flag, handle in error dispatch
4. Each command file — annotate `domain_bail!` calls with the appropriate error code
5. Success path — optionally emit `{"ok": true, ...}` for mutations when `--json`

### Backward compatibility

- Without `--json`: zero change. Same stderr text, same exit codes.
- `domain_bail!("message")` without code defaults to `Validation` (most common)
- Existing `ready --json` behavior preserved (it already emits JSON on success)

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
