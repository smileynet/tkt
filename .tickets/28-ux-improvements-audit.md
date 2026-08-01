---
id: "28"
title: "UX improvements: audit command, close flow polish, output consistency"
status: open
blocked_by: ["34", "30", "31", "32", "33"]
---

# UX improvements: tracking ticket

Parent ticket for the UX improvement pass. Closes when all children are done.

## Children

- **#34** — `tkt audit` command (batch closure quality check)
- **#30** — `tkt close` shows newly unblocked tickets
- **#31** — Output consistency (action-result pattern with symbols)
- **#32** — `tkt ready` information hierarchy (blocked by #31)
- **#33** — `--quiet` / `-q` flag for scriptability (blocked by #31)

## Dependency graph

```
#27 (done) ──┬──→ #34 audit command
             ├──→ #30 close shows unblocked
             └──→ #31 output consistency ──┬──→ #32 ready hierarchy
                                           └──→ #33 quiet flag
                                    ↓
                              #28 (tracking — all children done)
```

## Design rationale

Research files in `.scratch/research/`:
- `cli-ux-patterns.md` — information hierarchy, progressive disclosure, cargo/gh/git patterns
- `audit-lint-prior-art.md` — npm/cargo/brew doctor command designs
- `jtbd-close-workflow.md` — cognitive closure, Zeigarnik effect, completion friction

## Acceptance criteria

- [ ] All children (#30–#34) are done
