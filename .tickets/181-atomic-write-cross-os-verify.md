---
id: "181"
title: "Verify atomic_write on Windows + Linux (fs::rename overwrite, temp cleanup, worktrees)"
status: open
blocked_by: ["178"]
priority: medium
validation_criteria:
  - "Confirm whether std::fs::rename can overwrite an existing file on the target Windows env (msvc); decide if the #[cfg(windows)] delete-first branch in core::atomic_write is necessary or can be removed, and update the AGENTS.md 'Windows rename cannot overwrite' constraint accordingly"
  - "Run the full test suite on Linux and Windows (incl. a Windows equivalent of test_close_write_failure_surfaces_os_error, which is currently #[cfg(unix)]) and confirm atomic_write behaves correctly on git worktrees and shared/virtual filesystems"
tags: ["cli"]
---

# Verify atomic_write on Windows + Linux (fs::rename overwrite, temp cleanup, worktrees)

## What to build

TBD

## Acceptance criteria

- [ ] TBD
