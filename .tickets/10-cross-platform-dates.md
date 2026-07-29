---
id: "10"
title: "deterministic cross-platform date generation"
status: open
blocked_by: []
---

# Deterministic cross-platform date generation

## What to build

`chrono_date` in `cli.rs` shells out to Unix `date` and falls back to parsing Windows `%date%` with locale-dependent slicing. This is fragile (locale-dependent, can produce "UNDATED") and adds a subprocess dependency.

**Fix:** Use Rust's standard library or a zero-dep approach to get the current date in ISO 8601 format (YYYY-MM-DD). Options:
- `std::time::SystemTime` + manual formatting (no deps, ~10 lines)
- Or accept one small dep like `time` crate if the project already uses it

### Changes needed

1. Replace `chrono_date()` with pure-Rust date formatting
2. Remove the shell-out to `date` / `%date%`
3. Ensure the format matches what Python tkt produces (ISO 8601: YYYY-MM-DD)
4. Remove the "UNDATED" fallback path

## Acceptance criteria

- [ ] `close` command appends correct ISO date on all platforms
- [ ] No subprocess call for date generation
- [ ] No locale dependency
- [ ] Works on Windows, Linux, macOS
- [ ] Existing close tests pass with real dates (not "UNDATED")
