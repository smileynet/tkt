---
id: "02"
title: "preflight race check + remote ticket scanning"
status: done
blocked_by: ["07"]
---

# Preflight race check + remote ticket scanning

## What to build

The Python tkt scans remote ticket filenames before allocating IDs and validates ticket state before mutating. The Rust implementation only fetches and scans local files, making it vulnerable to same-second collisions and stale-state overwrites.

### Changes needed

1. **Remote ticket name scanning** — Add `git::remote_ticket_names(repo) -> Vec<String>` that runs `git ls-tree --name-only origin/main .tickets/` after fetch to get the authoritative list of allocated names without modifying the working tree.

2. **Use remote names in allocation** — `cmd_new` and `cmd_batch` should union local and remote filenames when computing `max_id` and checking for occupied IDs.

3. **Preflight state check for mutations** — Before `claim`, `close`, and `edit` write changes, fetch and reload the target ticket from the working tree (post-fetch) to verify preconditions still hold. Specifically:
   - `claim`: target is still `open`
   - `close`: target is still `in_progress` (or `open` if allowed)
   - `edit`: target still exists and has expected current state

4. **Batch collision recovery** — When `batch` push is rejected, undo the commit, re-fetch, rescan remote names, reallocate the entire batch with new IDs, and retry (matching Python behavior).

5. **Transaction-level retry for mutations** — Replace blind `pull --rebase` with: fetch → reload affected ticket → recheck preconditions → recompute mutation → commit → push.

### Out of scope (separate tickets)

- Timeout/non-interactive safeguards (ticket 07 covers push safety)
- Full concurrent test suite (ticket 13)

## Acceptance criteria

- [x] `git ls-tree` used to scan remote ticket names after fetch
- [x] `new` allocation uses union of local + remote filenames
- [x] `batch` allocation uses union of local + remote filenames
- [x] `batch` on push rejection: undoes commit, refetches, reallocates IDs, retries
- [x] `claim` verifies target is still `open` after fetch (before writing)
- [x] `close` verifies target is still claimable/in_progress after fetch
- [x] `edit` verifies target still exists after fetch
- [x] Integration test: two clones, competing allocation → no ID collision
- [x] Integration test: stale claim (ticket closed remotely) → fails cleanly
