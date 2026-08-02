---
id: "33"
title: "--quiet/-q flag: suppress confirmations, emit only essential data"
status: done
blocked_by: ["31"]
---

# --quiet/-q flag: suppress confirmations, emit only essential data

## What to build

Add a `-q` / `--quiet` global flag that suppresses confirmation messages, emitting only the essential datum for scripting.

### Behavior per command

| Command | Normal output | Quiet output |
|---------|--------------|--------------|
| `tkt new` | `✓ created 01 foo (pushed)` | `01` (just the ID) |
| `tkt batch` | multiple `✓ created...` lines | one ID per line |
| `tkt claim` | `✓ claimed 01 foo → in_progress` | (nothing) |
| `tkt close` | `✓ closed 01 foo ...` | (nothing) |
| `tkt edit` | `✓ edited 01 foo (fields)` | (nothing) |
| `tkt ready` | full frontier display | one ID per line (bare) |

### Use cases

```bash
# Create and immediately claim
tkt new auth --title "Auth" -q | xargs tkt claim

# Script that processes all frontier IDs
for id in $(tkt ready -q); do tkt claim $id; done
```

### Implementation

- Global flag on the `Cli` struct: `#[arg(short, long, global = true)] quiet: bool`
- Pass to each command function, suppress non-essential output when true
- Errors (exit 1, exit 2) still print to stderr regardless of quiet

### Stderr vs stdout contract

When `-q` is active:
- stdout: only essential data (IDs, JSON Lines for query)
- stderr: errors always print
- Confirmations, AC counts, unblocked notices suppressed

## Deletion test

Without this, scripting with tkt requires parsing human-readable output (fragile). The `-q` flag enables robust composability.

## Acceptance criteria

- [ ] `-q` / `--quiet` suppresses confirmation output
- [ ] `tkt new -q` prints only the allocated ID
- [ ] `tkt ready -q` prints one ID per line (no headers, no flags)
- [ ] Errors still print to stderr in quiet mode
- [ ] Integration test: `tkt new ... -q` output is a bare ID
