---
id: "26"
title: "crew-wide adoption: install in all projects, add to steering"
status: done
blocked_by: ["24", "25"]
---

# Crew-wide adoption: install in all projects, add to steering

## What to build

Roll out tkt as the standard ticket management tool across all crew projects. Update steering, AGENTS.md files, and tool-installation guides to reference the Rust binary.

### Steps

1. **Update tool-installation.md** — add tkt to the required tools table with install command (`cargo install --path D:/code/tkt` or from GitHub release)
2. **Update frontier-work.md** — confirm it references `tkt ready` (already does, but verify)
3. **Add .tickets/ to projects that don't have it** — kc2-ui-workshop, any other active projects
4. **Enable telemetry in crew config** — add `TKT_TELEMETRY=on` to crew shell profiles (opt-in by choice, not by default for external users)
5. **Add `tkt validate` to pre-commit/CI** — each project that has .tickets/ should validate on push
6. **Document in crew onboarding** — new sessions should see tkt available and know to use `tkt ready` for frontier work

### Deletion test

Without formal adoption, tkt remains a single-project tool instead of crew infrastructure.

## Acceptance criteria

- [x] tool-installation.md lists tkt with install command
- [x] All active crew projects have .tickets/ initialized
- [x] `tkt validate` runs in CI or pre-commit for projects with tickets
- [x] Telemetry enabled in crew shell profiles
- [x] Frontier-work steering references tkt correctly
- [x] Agent sessions can discover and use tkt without manual setup
