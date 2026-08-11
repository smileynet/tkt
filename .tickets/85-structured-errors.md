---
id: "85"
title: "Structured error envelopes for agent-parseable failures"
status: open
blocked_by: []
priority: medium
---

# Structured error envelopes for agent-parseable failures

## Context

Currently tkt errors are human-readable text on stderr. Agents must regex-parse error messages to understand what went wrong. A structured error envelope lets agents distinguish error types programmatically and decide on recovery actions.

## What to build

When `--json` is active (or a global `--output json` flag), errors emit structured JSON:

```json
{"ok": false, "error": "not_found", "message": "Ticket 99 not found", "exit_code": 1}
{"ok": false, "error": "conflict", "message": "Push rejected: ticket 03 claimed by another session", "exit_code": 1}
{"ok": false, "error": "validation", "message": "Invalid priority 'critical' — expected urgent|high|medium|low", "exit_code": 2}
```

Error types:
- `not_found` — ticket ID doesn't exist
- `conflict` — push race / claim conflict
- `validation` — invalid input (bad priority, bad slug, etc.)
- `cycle` — dependency cycle detected
- `already_done` — trying to close an already-closed ticket
- `blocked` — trying to close a ticket with unsatisfied deps
- `io` — filesystem or git subprocess failure

## Acceptance criteria

- [ ] Errors emit JSON envelope when --json flag is active
- [ ] Error type field uses a fixed vocabulary (enumerable)
- [ ] Human-readable message still present in envelope
- [ ] Exit codes consistent: 1=domain error, 2=operational error
- [ ] Backward-compatible: without --json, stderr text unchanged
- [ ] At least 5 distinct error types covered

# Structured error envelopes for agent-parseable failures

## What to build

TBD

## Acceptance criteria

- [ ] TBD
