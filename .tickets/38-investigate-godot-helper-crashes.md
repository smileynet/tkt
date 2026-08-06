---
id: "38"
title: "investigate godot-helper crashes (6 exit-2 on read commands)"
status: open
blocked_by: []
priority: high
---

# Investigate godot-helper crashes (6 exit-2 on read commands)

## Observed

Telemetry shows 6 crashes (exit code 2) in godot-helper, primarily on read-only commands:
- `ready` × 4 (Aug 1, Aug 4 ×2)
- `edit` × 1
- `query` × 1

Exit code 2 = operational crash (I/O, git, parse error). Read commands crashing suggests a malformed ticket file that fails to parse.

## Investigation steps

1. Run `TKT_DEBUG=1 tkt ready` in godot-helper to reproduce
2. Run `tkt validate` to identify unparseable files
3. Check if any .tickets/*.md files have invalid frontmatter (missing required field, bad YAML, encoding issues)
4. If parse error: improve error resilience (skip unparseable files in read commands with warning instead of crashing)

## Design question

Should `tkt ready` and `tkt query` crash (exit 2) on a single unparseable file, or should they skip it with a warning and continue? Currently `load_corpus()` fails on first parse error. `tkt validate` already handles this gracefully (collects parse errors as findings). The read commands should probably do the same — degrade gracefully on bad files.

## Acceptance criteria

- [ ] Root cause identified for godot-helper crashes
- [ ] Fix applied (either fix the malformed file, or make read commands resilient)
- [ ] If resilience fix: read commands skip unparseable files with stderr warning
- [ ] Regression test for graceful degradation on malformed files
