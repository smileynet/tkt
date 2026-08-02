---
id: "35"
title: "Streamline tkt close workflow (--resolution, --check-all flags)"
status: done
blocked_by: []
priority: high
---

# Streamline tkt close workflow (--resolution, --check-all flags)

## Problem

The current `tkt close <id>` workflow requires 3 separate steps after the command:
1. `tkt close N` — appends "TBD" resolution stub, warns about unchecked boxes
2. Manually edit the file to check all `- [ ]` boxes → `- [x]`
3. Manually edit to replace "TBD" with the actual resolution text
4. `git add` + `git commit` + `git push`

In a productive session, this happens 6-10 times. The manual edit step breaks flow — you leave the terminal, open the file, make regex-like edits, save, return.

## Observed session log (2026-08-02, lacrosse-bosse-helper)

```
# Repeated 6+ times this pattern:
> tkt close 50
  "warning: 4 unchecked acceptance box(es)"
  "closed 50-qr-measure-play-sizes.md (dated Resolution stub appended)"

# Then manually:
> (edit file to check boxes and fill resolution)
> git add .tickets/50-qr-measure-play-sizes.md
> git commit -m "..."
> git push
```

The agent (kiro-cli) handling this had to:
- Read the file to find the AC section offset
- Use strReplace to check boxes and fill resolution
- Commit separately

This happened for tickets 50, 51, 52, 53, 54, 55, 57, 58, 59 in one session.

## Proposed solution

### `--resolution` flag
```bash
tkt close 50 --resolution "QR feasibility confirmed. All plays fit within capacity."
```
Fills the Resolution section directly instead of "TBD".

### `--check-all` flag
```bash
tkt close 50 --check-all
```
Converts all `- [ ]` to `- [x]` in the AC section. The existing "warn on unchecked" behavior becomes the default (no flag), and `--check-all` suppresses the warning by checking them.

### Combined
```bash
tkt close 50 --check-all --resolution "Feasibility confirmed."
```

One command, one commit, one push. No file editing needed.

## Edge cases

- `--check-all` with some boxes intentionally unchecked: don't use the flag, check manually (current behavior preserved)
- `--resolution` with multi-line text: accept heredoc or quote-delimited string
- `--resolution` without `--check-all`: should still warn about unchecked boxes (resolution doesn't imply all AC met)

## Acceptance criteria

- [x] `tkt close N --resolution "text"` fills resolution instead of TBD
- [x] `tkt close N --check-all` checks all AC boxes
- [x] Both flags work together
- [x] Existing behavior (no flags) unchanged
- [x] Warning still fires when boxes unchecked and `--check-all` not passed
