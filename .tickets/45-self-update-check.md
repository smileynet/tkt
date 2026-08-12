---
id: "45"
title: "self-update check: notify when a newer version is available"
status: in_progress
blocked_by: ["01"]
validation_criteria: 
  - "Update check runs on invocation, at most once per 24 hours"
  - "Prints notice to stderr when newer version exists"
  - "Check result cached with timestamp"
  - "3-second timeout, silent on network errors"
  - "Disabled via env var and quiet mode"
  - "Does not affect exit code or stdout"
---

# Self-update check: notify when a newer version is available

## What to build

On invocation, tkt checks if a newer version is available and prints a one-line notice to stderr. Enabled by default, configurable to disable.

### Behavior

```
$ tkt ready
Ready (3):
  01  Set up release pipeline
  ...

(tkt 0.2.0 available — run `cargo install tkt` to update)
```

- Check runs at most once per 24 hours (cached timestamp in data dir)
- Network timeout: 3 seconds max — if check fails, silently skip
- Never blocks or delays the primary command output
- Notice goes to stderr (doesn't pollute piped/JSON output)

### Check mechanism

Query the crates.io API or GitHub releases API for the latest version. Compare with `env!("CARGO_PKG_VERSION")`. If newer exists, print notice.

```
GET https://crates.io/api/v1/crates/tkt
→ parse .crate.max_stable_version
```

Or for pre-publish (GitHub only):
```
GET https://api.github.com/repos/smileynet/tkt/releases/latest
→ parse .tag_name
```

### Configuration

```toml
# ~/.config/tkt/consent.toml (existing file)
[update]
check = true          # default: true
last_check = "2026-08-06"  # written automatically
```

Disable via:
- Config file: `check = false`
- Env var: `TKT_UPDATE_CHECK=0`
- Flag: `--no-update-check` (global, for scripts)

### Constraints

- Must not add latency to the happy path (fire-and-forget or cached)
- Must respect offline environments (CI, air-gapped systems — auto-disable when no network)
- Blocked by #01 (needs a published version to check against)

## Acceptance criteria

- [ ] Update check runs on invocation, at most once per 24 hours
- [ ] Prints one-line notice to stderr when newer version exists
- [ ] Check result cached with timestamp (no repeated network calls)
- [ ] 3-second timeout, silent failure on network errors
- [ ] Disabled via config file, env var, or global flag
- [ ] Enabled by default
- [ ] Does not affect exit code or stdout
- [ ] Works with crates.io API (or GitHub releases pre-publish)
