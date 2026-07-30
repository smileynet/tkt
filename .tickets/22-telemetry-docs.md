---
id: "22"
title: "TELEMETRY.md transparency document"
status: done
blocked_by: ["18", "19", "20", "21"]
---

# TELEMETRY.md transparency document

## What to build

Write a public TELEMETRY.md documenting exactly what tkt collects, where it stores data, how to opt in/out, and how to inspect/delete local data. This is the trust contract with users.

### Contents

1. What is collected (full field list with examples)
2. What is never collected (explicit exclusions)
3. Where data is stored (paths per platform)
4. How to opt in / opt out (commands + env vars)
5. How to inspect local data (`tkt telemetry --show`)
6. How to delete local data (`tkt telemetry --clear`)
7. Storage limits (rotation policy, max size)
8. Whether data leaves the machine (v1: never — local only)

### Deletion test

Without this document, users can't make an informed consent decision. Trust requires transparency.

## Acceptance criteria

- [x] TELEMETRY.md exists in repo root
- [x] Documents all collected fields with example values
- [x] Documents all excluded data categories
- [x] Documents storage paths for Linux, macOS, Windows
- [x] Documents all opt-in/out mechanisms
- [x] Documents storage limits and rotation policy
- [x] States clearly that v1 is local-only (no upload)
- [x] Referenced from README.md
