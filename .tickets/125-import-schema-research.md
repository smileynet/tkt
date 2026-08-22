---
id: "125"
title: "Research and build import schemas for Jira, Linear, Beads, GitLab, and other ticket systems"
status: backlog
blocked_by: ["77"]
priority: medium
validation_criteria:
  - "schema mappings documented for at least 3 external systems"
  - "migrate.toml templates exist for each researched system"
---

# Research and build import schemas for Jira, Linear, Beads, GitLab, and other ticket systems

## Problem

`tkt migrate` (ticket 77) establishes the migration infrastructure with `tk` as the first adapter. To reduce switching costs and enable adoption, tkt needs import compatibility with popular ticket systems that teams are already using.

## What to build

Dispatch subagents to research each system's data model, then produce `migrate.toml` schema mappings that `tkt migrate --map` can consume.

### Systems to research

| System | Export method | Priority |
|--------|-------------|----------|
| **Jira** | REST API / CSV export / JSON export | High (most enterprise teams) |
| **Linear** | GraphQL API / CSV export | High (developer-focused teams) |
| **GitLab Issues** | API / CSV export | Medium (self-hosted teams) |
| **GitHub Issues** | `gh` CLI (see ticket 86) | Medium (already scoped) |
| **Beads** | File-based (local) | Medium (similar to tkt) |
| **Trello** | JSON export | Low (less structured) |
| **Notion databases** | CSV/API export | Low (highly custom schemas) |
| **Plain TODO.md** | File parsing | Low (no standard schema) |

### For each system, produce:

1. **Schema analysis** — fields available, ID format, status values, dependency representation, priority levels
2. **Field mapping** — which fields map to tkt fields, which are dropped, which need value translation
3. **migrate.toml template** — ready-to-use mapping file for `tkt migrate --map`
4. **Import method** — how to get data out (API call, export button, file copy)
5. **Limitations** — what's lost in translation, bidirectional sync feasibility

### Deliverables

- `.memory/specs/import-schemas/` directory with one doc per system
- `skills/tkt/references/` templates for agent-assisted import
- Recommendations for which systems warrant built-in CLI adapters vs skill-assisted import

## Context

- **Depends on:** ticket 77 (migrate infrastructure must exist first)
- **Related:** ticket 86 (GitHub import — one-way pull, different from schema migration)
- **Architecture:** `migrate.toml` is the seam — agents research and produce the map, CLI applies it mechanically

## Acceptance criteria

- [ ] At least 3 systems researched with full schema analysis
- [ ] migrate.toml templates produced and validated against sample data
- [ ] Recommendations documented: which warrant built-in adapters vs skill-only
- [ ] Research findings stored in .memory/specs/import-schemas/
