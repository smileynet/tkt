---
id: "19"
title: "log rotation, retention, and session-aware cleanup"
status: open
blocked_by: ["18"]
---

# Log rotation, retention, and session-aware cleanup

## What to build

Add rotation and retention to the telemetry JSONL files so they don't grow unbounded. Cleanup must be session-aware — don't just delete by age, but understand that a "session" is a single CLI invocation and old sessions should be pruned as a unit.

### Design

1. **Per-project files** — `~/.local/share/tkt/telemetry/{project-slug}.jsonl` (one file per project, rotated independently)
2. **Size-based rotation** — when a project file exceeds 5MB, rotate to `{project-slug}.1.jsonl`, shift older files up, delete beyond max
3. **Max files per project** — keep 5 rotated files max (hard cap: 25MB per project)
4. **Max age** — delete rotated files older than 30 days regardless of count
5. **Startup cleanup** — on every tkt invocation, scan the telemetry directory and enforce retention rules before writing new events. Keep it fast (just stat calls, no parsing).
6. **Session-aware pruning** — when a project's total storage exceeds budget, identify the oldest complete sessions (by session ID boundaries) and remove whole sessions rather than arbitrary line counts. This preserves diagnostic coherence.
7. **Compression** — optionally gzip rotated files (`.jsonl.gz`). Implementation detail: if it adds complexity without clear benefit at our scale, skip for v1.

### Segmentation fields

Each JSONL line already has `session` and `project` from ticket #18. This ticket adds:
- Rotation logic that respects these boundaries
- A cleanup scan that groups records by session before deciding what to prune
- `tkt telemetry --show` output that reports storage per project

### Deletion test

Without rotation, the telemetry files grow indefinitely. Without session-awareness, truncation can split a diagnostic session in half, making it useless for debugging.

## Acceptance criteria

- [ ] Per-project JSONL files rotate at 5MB threshold
- [ ] Max 5 rotated files per project (configurable via const)
- [ ] Files older than 30 days are deleted on startup scan
- [ ] Startup cleanup completes in <50ms for typical directory (10-20 files)
- [ ] Session boundaries are preserved during pruning (no partial sessions)
- [ ] Total telemetry storage is bounded (documented max in TELEMETRY.md)
- [ ] Unit tests for rotation trigger, retention enforcement, session boundary detection
