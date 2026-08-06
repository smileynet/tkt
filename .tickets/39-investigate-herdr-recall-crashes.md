---
id: "39"
title: "investigate herdr/recall query crashes (exit 2 on query)"
status: done
blocked_by: []
---

# Investigate herdr/recall query crashes (exit 2 on query)

## Observed

Telemetry shows crashes on `tkt query` in two projects:
- `herdr` × 3 (all on Aug 1)
- `recall` × 1 (Aug 1)

Both projects had query attempts crash. Likely same root cause as #38 (malformed ticket files), but in different repos.

## Investigation steps

1. Check if herdr and recall have `.tickets/` directories with valid files
2. Run `tkt validate` in each to identify parse failures
3. If the projects don't have tickets but an agent tried to use tkt anyway: the error message should be clear ("no .tickets/ directory" is exit 1 domain error, not exit 2 crash — so something else is happening)

## Possible causes

- Git repo not fully initialized (no initial commit — `repo_root()` might fail)
- .tickets/ exists but contains non-ticket .md files that fail to parse
- Permission issues on the directory

## Acceptance criteria

- [ ] Root cause identified for herdr and recall crashes
- [ ] Fix applied or documented (if it's a project setup issue vs tkt bug)
