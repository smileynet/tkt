---
id: "44"
title: "implement multi-level priority system (urgent/high/medium/low)"
status: open
blocked_by: []
---

# Implement multi-level priority system (urgent/high/medium/low)

## Context

tkt currently has a binary priority: `high` (jumps the frontier) or nothing. Real-world usage (godot-helper) shows people writing `priority: low` expecting it to work. The telemetry crash investigation (#38) revealed the enum was too strict.

Research into priority systems across Linear, Jira, Taskwarrior, and Todoist found universal convergence on 4 levels plus "none". The frontier ordering research confirmed that tkt's current algorithm (priority bucket + ID sort within the ready set) is correct — it just needs more buckets.

## Design

### Priority levels

| Level | Frontmatter value | Frontier effect |
|-------|-------------------|-----------------|
| Urgent | `priority: urgent` | First bucket (above high) |
| High | `priority: high` | Second bucket |
| (none) | no priority field | Default bucket (same as medium) |
| Medium | `priority: medium` | Explicit default (same as no priority) |
| Low | `priority: low` | Last bucket (deprioritized) |

### Frontier sort order

Within the ready set: urgent → high → normal (no priority / medium) → low. Within each bucket: lowest ID first (existing behavior).

### Backward compatibility

- `priority: high` continues to work identically (jumps to second bucket, below urgent)
- Omitting priority still works (default/medium bucket)
- Unknown values are silently treated as default (lenient parsing, per #38 fix)

### Validation

`tkt validate` should warn on unknown priority values (not error). `tkt edit --priority` should accept: `urgent`, `high`, `medium`, `low`, or empty string to clear.

### Display

`tkt ready` shows:
- `[URGENT]` flag for urgent tickets
- `[HIGH]` flag for high tickets (existing)
- No flag for medium/default
- `[low]` (lowercase, dimmed if color) for low tickets

### What this does NOT include

- Urgency scoring (Taskwarrior-style composite scores) — overkill for <100 tickets
- Relative ordering within buckets (drag-and-drop) — doesn't map to file-based storage
- Due dates or time-based urgency — out of scope for v1
- Priority inheritance from dependents — complexity not justified yet

## Prior art (from research)

- **Linear:** 5 levels (No/Low/Medium/High/Urgent), explicitly refuses custom levels, "diminishing returns"
- **Taskwarrior:** 3 levels (H/M/L) fed into urgency coefficients (+6/+3.9/+1.8), combined with other factors
- **Todoist:** 4 levels (P1-P4), color-coded, filter-sortable
- **Frontier ordering:** Kahn's algorithm with priority queue — nodes enter queue when in-degree = 0, dequeue by priority. Exactly what tkt already does with 2 buckets.

## Acceptance criteria

- [ ] Priority enum supports: urgent, high, medium, low
- [ ] Unknown values treated as default (no crash, no error)
- [ ] Frontier sorts: urgent → high → default → low → by ID within each
- [ ] `tkt ready` shows appropriate flags ([URGENT], [HIGH], [low])
- [ ] `tkt edit --priority` accepts all valid values + empty to clear
- [ ] `tkt validate` warns on unknown priority values
- [ ] `tkt new --priority` accepts all valid values
- [ ] Backward compatible: existing `priority: high` tickets unchanged
- [ ] Unit tests for sort ordering with all priority levels
