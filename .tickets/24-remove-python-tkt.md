---
id: "24"
title: "uninstall Python tkt and clean up crew-research/tools/tkt"
status: done
blocked_by: ["23", "25"]
---

# Uninstall Python tkt and clean up crew-research/tools/tkt

## What to build

Remove the original Python tkt implementation from crew-research and ensure no project still references it. The Rust binary must be confirmed as a drop-in replacement before this happens (ticket #25 gates this).

### Steps

1. **Identify Python tkt location** — `crew-research/tools/tkt/` (the original implementation)
2. **Check for references** — grep across crew projects for imports, PATH entries, aliases, or scripts that reference the Python version
3. **Remove from crew-research** — delete `tools/tkt/` directory, update any `pyproject.toml` or `setup.py` if it was an installable package
4. **Remove from PATH** — if the Python version was installed via pip/pipx, run `pip uninstall tkt` or remove the symlink
5. **Verify no breakage** — confirm `which tkt` / `where tkt` points to the Rust binary in all environments
6. **Update AGENTS.md / steering** — if any crew steering references `crew-research/tools/tkt`, update to reference the standalone Rust repo

### Deletion test

The Python implementation is dead code once the Rust version is validated. Keeping it creates confusion about which tkt to use.

## Acceptance criteria

- [x] `crew-research/tools/tkt/` directory removed (or PR opened)
- [x] No remaining references to Python tkt in crew projects
- [x] `which tkt` / `where tkt` resolves to Rust binary on all machines
- [x] No pip/pipx installation of old tkt remains
- [x] Steering/AGENTS.md references updated
