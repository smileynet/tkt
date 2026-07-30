---
id: "25"
title: "test-run tkt in crew projects to confirm drop-in replacement"
status: done
blocked_by: ["22", "23"]
---

# Test-run tkt in crew projects to confirm drop-in replacement

## What to build

Install the Rust tkt binary and exercise it in 3+ crew projects to confirm it's a functional drop-in replacement for the Python version. Run with telemetry enabled and debug mode to validate those new features under real workloads.

### Test matrix

| Project | Has .tickets/ | Test commands | Expected |
|---------|:---:|---|---|
| tkt (self) | ✅ | ready, validate, query, close | Already dogfooding |
| game-research | ✅ | ready, validate, new, claim, close | Full lifecycle |
| shadowrun-sega | ✅ | ready, validate, query | Read-only smoke test |
| kc2-ui-workshop | ❓ | init .tickets/, new, ready | Fresh project test |

### Test procedure per project

1. `cd <project>` and confirm `tkt --version` shows Rust version
2. `TKT_DEBUG=1 tkt ready` — verify debug output is coherent
3. `tkt validate` — confirm no regressions from Python output format
4. `tkt query | head` — verify JSON Lines output matches expected schema
5. `tkt telemetry --enable` then run a few commands, then `tkt telemetry --status` — confirm events are being recorded
6. Check `~/.local/share/tkt/telemetry/{project}.jsonl` contains valid JSONL with correct session/project fields
7. Compare output format with any saved Python tkt output (if available)

### Success criteria

All commands produce identical output format and exit codes as the Python version. Telemetry records events correctly per-project. Debug mode shows useful diagnostic trace.

### Deletion test

Without real-world validation, we can't safely remove the Python version or adopt crew-wide.

## Acceptance criteria

- [x] tkt installed via `cargo install --path .` on test machine
- [x] `tkt ready` works correctly in 3+ projects
- [x] `tkt validate` produces correct findings in projects with known issues
- [x] `tkt query` JSON output matches documented schema
- [x] Telemetry records events with correct project slug and session ID
- [x] Debug mode (`TKT_DEBUG=1`) produces useful trace in real projects
- [x] No regressions in exit codes (0=success, 1=domain, 2=crash)
- [x] Log files appear in expected platform directory with correct segmentation
