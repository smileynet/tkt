---
id: "90"
title: "tkt open <id>: find URLs in ticket body and open in browser"
status: open
blocked_by: []
priority: low
---

# tkt open: find URLs in ticket body and open in browser

## Context

dstask's `open` command finds URLs in task notes and opens them in the browser. Tickets often reference PRs, docs, or design links — `tkt open <id>` saves the manual copy-paste.

## What to build

```bash
tkt open 03
# → Opening: https://github.com/smileynet/tkt/pull/42

tkt open 03 --list
# → URLs in ticket 03:
# →   1. https://github.com/smileynet/tkt/pull/42
# →   2. https://docs.example.com/deploy-guide
```

Behavior:
- Scans ticket body (below frontmatter) for URLs
- Opens the first URL found (or prompts if multiple with `--list`)
- Uses `xdg-open` (Linux), `open` (macOS), `start` (Windows)
- Error if no URLs found in ticket

## Acceptance criteria

- [ ] `tkt open <id>` opens first URL found in ticket body
- [ ] `tkt open <id> --list` shows all URLs numbered
- [ ] Cross-platform: xdg-open / open / start
- [ ] Error message if no URLs in ticket
- [ ] Error message if ticket not found

# tkt open <id>: find URLs in ticket body and open in browser

## What to build

TBD

## Acceptance criteria

- [ ] TBD
