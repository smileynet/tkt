---
id: "21"
title: "debug mode: TKT_DEBUG=1 prints structured trace to stderr"
status: open
blocked_by: ["18"]
---

# Debug mode: TKT_DEBUG=1 prints structured trace to stderr

## What to build

When `TKT_DEBUG=1` is set, tkt prints a structured diagnostic trace to stderr showing exactly what it's doing. This is for diagnosing issues in real-time (not persisted to the telemetry file — that's separate).

### Behavior

- `TKT_DEBUG=1` → print all events to stderr in a human-readable format
- `TKT_DEBUG=json` → print all events to stderr as JSONL (machine-parseable)
- Does NOT require telemetry consent (debug mode is ephemeral)
- Does NOT affect stdout (commands still produce normal output)
- Does NOT persist anything (unless telemetry is also enabled separately)

### Output format (human)

```
[tkt:debug] session=01J6XYZABC project=tkt cmd=claim
[tkt:debug] git fetch origin (0.3s)
[tkt:debug] corpus loaded: 17 tickets (4 open, 2 in_progress, 11 done)
[tkt:debug] remote check: .tickets/03-parity-input-validation.md status=open
[tkt:debug] set_field status=in_progress
[tkt:debug] git add .tickets/03-parity-input-validation.md
[tkt:debug] git commit "chore(tickets): claim 03"
[tkt:debug] git push (0.8s)
[tkt:debug] exit=0 duration=1.2s
```

### Integration points

Instrument the key operations:
- Git subprocess calls (command + duration)
- Corpus loading (count by status)
- Remote state checks
- Field mutations
- File writes
- Push attempts and retries
- Exit code and total duration

### Deletion test

Without debug mode, diagnosing tkt issues requires adding println! statements, rebuilding, and removing them. Debug mode makes tkt self-diagnosing.

## Acceptance criteria

- [ ] `TKT_DEBUG=1` prints human-readable trace to stderr
- [ ] `TKT_DEBUG=json` prints JSONL to stderr
- [ ] Normal stdout output is unchanged
- [ ] Debug output includes: session ID, project, command, git calls, corpus stats, exit code, duration
- [ ] Debug mode works without telemetry consent
- [ ] No performance impact when TKT_DEBUG is unset (zero-cost when disabled)
- [ ] Integration test: run with TKT_DEBUG=1, verify stderr contains expected trace lines
