---
id: "85"
title: "Structured error envelopes for agent-parseable failures"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "tkt -o json close 999 emits JSON error envelope to stderr"
  - "envelope has ok, error.kind, error.message, error.hint fields"
  - "without -o json, stderr text output unchanged"
  - "tkt capabilities (or schema) declares error kinds with exit codes"
  - "cargo test passes"
---

# Structured error envelopes for agent-parseable failures

## Problem

tkt errors are human-readable text on stderr (`tkt: ✗ Ticket 99 not found`). Agents must regex-parse these to understand what went wrong. A structured envelope lets agents distinguish error types programmatically and decide recovery actions without brittle string matching.

## Research Findings

Cloned and studied:
- **clispec v0.3** (clispec.dev) — emerging standard for agent-friendly CLIs. Defines error envelopes as last line of stderr, declares error kinds with exit codes and retryable flags in schema.
- **octo-cli** (Mininglamp-OSS) — Go CLI designed for AI agents. Uses `{ok: false, error: {type, code, message, hint, detail}}` envelope to stderr.
- **axocli** (axodotdev) — Rust CLI lib. Emits JSON error to stdout AND human error to stderr simultaneously.

Key consensus across sources:
1. **Errors on stderr** — stdout is for data only (clispec Principle 3)
2. **Error envelope as last line of stderr** when structured output requested (clispec §Errors)
3. **Fixed error kind vocabulary** declared in schema with exit_code + retryable (clispec, octo-cli)
4. **Hint field** for actionable remediation (clispec, octo-cli, Stripe)
5. **Exit code is primary machine signal** — even without JSON mode (clispec)
6. **`-o json` is the canonical flag** (clispec prefers `--output`/`-o` over `--json`)

## What to build

### Global `-o json` flag (clispec-aligned)

Replace `ready --json` with a global `--output`/`-o` flag. When `-o json`:
- **Success**: commands emit JSON to stdout
- **Errors**: emit JSON envelope as last line of stderr
- **Human text still on stderr**: both JSON and human error emitted (axocli pattern)

```bash
tkt -o json close 999
# stderr: {"ok":false,"error":{"kind":"not_found","message":"no ticket with id \"999\"","hint":"check tkt query for valid IDs"},"exit_code":1}
# exit code: 1

tkt -o json claim 01
# stdout: {"ok":true,"result":"claimed 01 auth-system (→ in_progress)","changed":true}
# exit code: 0
```

### Error envelope schema (clispec + octo-cli hybrid)

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

Fields:
- `ok` (bool) — false for errors, true for success
- `error.kind` (string) — stable identifier from declared vocabulary
- `error.message` (string) — human-readable detail (mutable, not a contract)
- `error.hint` (string, optional) — actionable remediation
- `exit_code` (int) — duplicated from process exit for when envelope is captured separately

### Error kind vocabulary (clispec-style declaration)

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

### Success envelope (mutations only)

```json
{
  "ok": true,
  "result": "claimed 01 auth-system (→ in_progress)",
  "changed": true
}
```

`changed` (bool) — clispec/Terraform convention: true when the command did work, false when state was already correct.

### Schema declaration (tkt capabilities)

Update `tkt capabilities` to declare error kinds per clispec:

```json
{
  "errors": [
    {"kind": "not_found", "exit_code": 1, "retryable": false, "description": "Ticket does not exist"},
    {"kind": "conflict", "exit_code": 1, "retryable": true, "description": "Push race or claim conflict"}
  ]
}
```

## Implementation

### Approach (from octo-cli pattern, adapted for Rust)

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
2. `src/commands/common.rs` — update `domain_bail!`: `domain_bail!(NotFound, "msg")` or `domain_bail!(NotFound, "msg", hint: "check IDs")`
3. `src/cli.rs` — replace `--json` on ready with global `-o`/`--output` flag; handle envelope in error dispatch
4. Each command file — annotate `domain_bail!` calls with error kind + optional hint
5. Success envelope — emit to stdout for mutations when `-o json`
6. `src/commands/capabilities.rs` — add `errors` array to schema output
7. Backward compat: `ready --json` still works (aliased)

### Backward compatibility

- Without `-o json`: zero change. Same stderr text, same exit codes.
- `ready --json` preserved as hidden alias
- `domain_bail!("message")` without explicit kind defaults to `Validation`

## Context

- `src/main.rs` — DomainError struct (line 21)
- `src/cli.rs` — error handler (line 424), global flags
- `src/commands/common.rs` — domain_bail! macro (line 10)
- `.references/clispec/` — clispec schema and test fixtures (error kinds, exit codes)
- `.references/octo-cli/internal/output/` — envelope.go, errors.go (ExitError pattern)
- `.references/axocli/src/lib.rs` — json_errors mode (dual output pattern)

## Acceptance criteria

- [ ] Global `-o json` flag available on all commands
- [ ] Errors emit JSON envelope to stderr (last line) when -o json active
- [ ] Error kind field uses fixed vocabulary (8 types declared)
- [ ] Hint field present when remediation is deterministic
- [ ] Exit codes consistent: 1=domain, 2=operational
- [ ] Without -o json, stderr text output unchanged (backward compatible)
- [ ] Mutation success emits `{"ok": true, "result": "...", "changed": ...}` to stdout
- [ ] `tkt capabilities` declares error kinds with exit_code + retryable
- [ ] `ready --json` still works (backward compat alias)

## Out of scope

- Full clispec v0.3 schema command (separate ticket — would declare all commands, effects, cardinality)
- Pagination (no unbounded collections in tkt currently)
- `--output yaml`/`--output text` (just json for now; text is the default)
- Auto-detect piped output (future clispec alignment)
