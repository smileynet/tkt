---
id: "109"
title: "tkt validate-all: cross-project health check with discovery"
status: open
blocked_by: []
priority: medium
validation_criteria: 
  - "tkt doctor <path> already provides this functionality"
---

# tkt validate-all: cross-project health check with discovery

## Problem

Running `tkt validate` across all projects required a manual shell loop repeated 3+ times this session. No built-in way to health-check all repos at once, and no visibility into which projects don't use tkt at all.

## What to build

```bash
tkt validate-all ~/code          # scan all git repos under path
tkt validate-all                  # default: current directory's parent (or ~)
tkt validate-all ~/code --strict  # pass --strict to each validate
tkt validate-all ~/code -o json   # JSON Lines output per project
```

### Behavior

1. **Discover** — recursively find directories containing `.tickets/` under the given path (max depth configurable, default 2)
2. **Validate** — run `tkt validate` in each discovered project
3. **Flag non-tkt projects** — git repos without `.tickets/` get a separate "not using tkt" line (not an error, just visibility)
4. **Report** — per-project status with error/warning counts
5. **Exit code** — 0 if all pass, 1 if any project has errors

### Output (text mode)

```
~/code/tkt                    ✓ pass (0 errors, 0 warnings)
~/code/recall                 ✓ pass (0 errors, 2 warnings)
~/code/archwright             ✓ pass (0 errors, 0 warnings)
~/code/some-other-repo        · no .tickets/ (git repo, not using tkt)
~/code/non-git-dir            · skipped (not a git repo)

5 projects scanned: 3 pass, 0 fail, 2 not using tkt
```

### Output (JSON mode)

```json
{"path":"~/code/tkt","status":"pass","errors":0,"warnings":0}
{"path":"~/code/recall","status":"pass","errors":0,"warnings":2}
{"path":"~/code/some-other-repo","status":"no_tickets","is_git":true}
```

### Flags

| Flag | Effect |
|------|--------|
| `--strict` | Pass `--strict` to each validate (warnings become errors) |
| `--brief` | Only show failures and non-tkt repos |
| `--max-depth N` | Discovery depth (default: 2) |
| `-o json` | JSON Lines output |

## Implementation

Two options:

**A. Subcommand** — `tkt validate-all <path>` as a new Rust command. Reuses the existing validate logic internally. Clean but couples discovery to the binary.

**B. Script** — `tools/validate-all.sh <path>` that invokes `tkt validate` per project. Simple, composable, no Rust changes. Can be wired as `mise run validate-all`.

**Recommendation: Option A** — it's a natural extension of `tkt doctor <path>` (which already scans multiple projects). Could even be `tkt doctor <path> --validate` or a `--validate` flag on the existing doctor command. Check what doctor already does to avoid duplication.

## Context

- `src/commands/doctor.rs` — already scans a path for `.tickets/` dirs and reports per-project health
- The manual pattern we repeated: `for repo in ...; cd "$dir" && tkt validate 2>/dev/null | python3 ...`
- `tkt doctor <path>` exists — may be better to extend it than add a new command

## Acceptance criteria

- [ ] Can scan a given path for all projects with `.tickets/`
- [ ] Reports pass/fail per project with error/warning counts
- [ ] Flags git repos not using tkt (visibility, not error)
- [ ] Exits non-zero if any project has validation errors
- [ ] Accepts `--strict` to escalate warnings
- [ ] Works with `-o json` for machine consumption
- [ ] Discovery depth configurable (default 2)

## Out of scope

- Auto-fixing across projects (that's `tkt lint` per project)
- Cross-project dependency checking
- Installing tkt in discovered projects
