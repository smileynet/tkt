---
id: "85"
title: "Structured error envelopes for agent-parseable failures"
status: in_progress
blocked_by: []
priority: medium
validation_criteria:
  - "tkt -o json close 999 emits JSON error envelope to stderr"
  - "envelope has ok, error.kind, error.message, error.hint fields"
  - "without -o json, stderr text output unchanged"
  - "tkt capabilities declares error kinds with exit codes"
  - "cargo test passes"
---

# Structured error envelopes for agent-parseable failures

## Problem

tkt errors are human-readable text on stderr (`tkt: ✗ Ticket 99 not found`). Agents must regex-parse these to understand what went wrong. A structured envelope lets agents distinguish error types programmatically and decide recovery actions without brittle string matching.

## Research Summary

Studied three reference implementations:

| Source | Stars | Authority | Pattern adopted |
|--------|-------|-----------|-----------------|
| **octo-cli** (Mininglamp-OSS) | 510⭐ | High — production AI-agent CLI, active daily | Error envelope shape: `{ok, error: {type, code, message, hint}}` |
| **axocli** (axodotdev/cargo-dist) | 16⭐ | Medium — sound engineering, but lib abandoned since Dec 2025 | Dual output: JSON to stderr + human text to stderr simultaneously |
| **clispec.dev** (rvben) | 9⭐ | Low — solo author, zero external adoption, 4 months old | Ideas are sound but not an adopted standard; treat as design inspiration |

Additional sources informing the design:
- **gh CLI** (GitHub) — `-o json` global flag pattern, errors stay as text on stderr
- **kubectl** — Status JSON object on failure with `-o json`
- **MCP** (Anthropic) — two-layer error model: codes for orchestration, messages for LLM reasoning
- **Stripe/Google APIs** — small fixed error vocabulary, hint/remediation fields

## What to build

### Global `-o json` flag

Industry-standard pattern (gh, kubectl, terraform). Replace `ready --json` with a global `--output`/`-o` flag:

```bash
tkt -o json close 999
# stderr: {"ok":false,"error":{"kind":"not_found","message":"no ticket with id \"999\"","hint":"check tkt query for valid IDs"},"exit_code":1}
# exit code: 1

tkt -o json claim 01
# stdout: {"ok":true,"result":"claimed 01 auth-system (→ in_progress)","changed":true}
# exit code: 0
```

### Error envelope (inspired by octo-cli's ExitError)

```json
{
  "ok": false,
  "error": {
    "kind": "not_found",
    "message": "no ticket with id \"999\"",
    "hint": "check tkt query for valid IDs"
  },
  "exit_code": 1
}
```

- Emitted as **last line of stderr** when `-o json` active
- Human text still printed to stderr regardless (axocli dual-output pattern)
- `error.kind` is stable (machine contract); `error.message` is mutable (human-facing)

### Error kind vocabulary

| Kind | Exit | Retryable | Description |
|------|------|-----------|-------------|
| `not_found` | 1 | false | Ticket/resource doesn't exist |
| `already_done` | 1 | false | Ticket already closed |
| `conflict` | 1 | true | Push race / claim lost — retry with new state |
| `gate_failed` | 1 | false | Quality gate blocked operation |
| `validation` | 1 | false | Invalid input (bad priority, slug, status) |
| `cycle` | 1 | false | Dependency cycle detected |
| `io` | 2 | false | Filesystem or git subprocess failure |
| `parse` | 2 | false | Ticket file couldn't be parsed |

Design rationale: 8 codes mapping to 4 recovery strategies (retry, fix input, escalate, give up). Only `conflict` is retryable — matches tkt's push-race retry semantics.

### Success envelope (mutations)

```json
{
  "ok": true,
  "result": "claimed 01 auth-system (→ in_progress)",
  "changed": true
}
```

`changed` (bool) — true when the command modified state, false when already in desired state. Useful for agent idempotency checks.

### Capabilities declaration

Update `tkt capabilities` to declare error kinds:

```json
{
  "errors": [
    {"kind": "not_found", "exit_code": 1, "retryable": false},
    {"kind": "conflict", "exit_code": 1, "retryable": true},
    ...
  ]
}
```

## Implementation

### ErrorKind enum + DomainError struct (from octo-cli's ExitError pattern)

```rust
#[derive(Debug, Clone, Copy)]
pub enum ErrorKind {
    NotFound,
    AlreadyDone,
    Conflict,
    GateFailed,
    Validation,
    Cycle,
    Io,
    Parse,
}

impl ErrorKind {
    pub fn as_str(&self) -> &'static str { ... }
    pub fn exit_code(&self) -> i32 { match self { Io | Parse => 2, _ => 1 } }
    pub fn retryable(&self) -> bool { matches!(self, Conflict) }
}

pub struct DomainError {
    pub kind: ErrorKind,
    pub message: String,
    pub hint: Option<String>,
}
```

### Changes needed

1. `src/main.rs` — expand `DomainError` to `{ kind, message, hint }`
2. `src/commands/common.rs` — update `domain_bail!`: `domain_bail!(NotFound, "msg")` or with hint
3. `src/cli.rs` — add global `-o`/`--output` flag; emit envelope in error handler
4. Each command file — annotate `domain_bail!` calls with error kind + optional hint
5. Success envelope — emit to stdout for mutations when `-o json`
6. `src/commands/capabilities.rs` — add `errors` array
7. Backward compat: `ready --json` still works (aliased to `-o json ready`)

### Backward compatibility

- Without `-o json`: zero change. Same stderr text, same exit codes.
- `ready --json` preserved as alias
- `domain_bail!("message")` without explicit kind defaults to `Validation`

## Context

- `src/main.rs` — DomainError struct (line 21)
- `src/cli.rs` — error handler (line 424), global flags
- `src/commands/common.rs` — domain_bail! macro (line 10)
- `.references/octo-cli/internal/output/envelope.go` — success/error envelope implementation
- `.references/octo-cli/internal/output/errors.go` — ExitError struct with Type/Code/Message/Hint
- `.references/axocli/src/lib.rs` — dual json+human error output (report_error fn)

## Acceptance criteria

- [ ] Global `-o json` flag available on all commands
- [ ] Errors emit JSON envelope as last line of stderr when -o json active
- [ ] Error kind field uses fixed vocabulary (8 types)
- [ ] Hint field present when remediation is deterministic
- [ ] Exit codes consistent: 1=domain, 2=operational
- [ ] Without -o json, stderr text output unchanged
- [ ] Mutation success emits `{"ok": true, "result": "...", "changed": ...}` to stdout
- [ ] `tkt capabilities` declares error kinds with exit_code + retryable
- [ ] `ready --json` backward compat alias works

## Out of scope

- Full clispec v0.3 schema command (aspirational standard, no adoption yet)
- Pagination (tkt has no unbounded collections)
- Auto-detect piped output format
- `--output yaml`/`--output text` variants
