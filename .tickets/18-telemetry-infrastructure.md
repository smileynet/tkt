---
id: "18"
title: "telemetry infrastructure: consent, session tracking, JSONL sink"
status: open
blocked_by: ["17"]
priority: high
---

# Telemetry infrastructure: consent, session tracking, JSONL sink

## What to build

Add a telemetry module that records structured events to a local JSONL file. This is the foundation for both usage analytics and debug logging.

### Components

1. **Consent system** — check hierarchy: `DO_NOT_TRACK=1` > `TKT_TELEMETRY=off` > `CI=true` > config file `~/.config/tkt/consent.toml`. Default: disabled.
2. **Session ID** — generate a ULID at process start. Included in every event record. Enables grouping all log lines from one CLI invocation.
3. **Project identification** — derive project slug from git repo root directory name. Enables per-project filtering.
4. **Event structure** — each JSONL line: `{"ts", "session", "project", "cmd", "level", "msg", "exit_code", "duration_ms", "version", "os", "arch"}`
5. **JSONL file sink** — append-only to `~/.local/share/tkt/telemetry/{project-slug}.jsonl` (Windows: `%APPDATA%\tkt\telemetry\`). Use platform-appropriate dirs via the `dirs` crate.
6. **Never block** — file I/O is synchronous but fast (single append). If write fails, swallow silently.

### What NOT to collect

- File paths, ticket content, git URLs
- Environment variable values
- Command argument values
- Full error messages (only error type/category)

### Deletion test

Without this, there's no local audit trail and no foundation for the debug mode, telemetry subcommand, or log rotation features.

## Acceptance criteria

- [ ] Consent check function respects the full hierarchy (env vars > config file > default off)
- [ ] Session ULID generated once per process, included in all events
- [ ] Project slug derived from repo root dirname
- [ ] Events appended as JSONL to platform-appropriate data directory
- [ ] Event schema includes: ts, session, project, cmd, level, msg, version, os, arch
- [ ] Write failures are swallowed silently (never surface to user)
- [ ] No new runtime dependencies beyond `dirs` (for platform paths) and `ulid` or equivalent
- [ ] Unit tests for consent hierarchy, event serialization, project slug derivation
