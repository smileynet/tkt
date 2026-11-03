---
id: "193"
title: "[labels.*] config: parse single-select + closed-value namespaces"
status: open
blocked_by: []
priority: high
validation_criteria:
  - "config.rs parses [labels.<ns>] tables with single_select bool and values list"
  - "Unknown/malformed [labels.*] entries produce a clear config error"
  - "Existing configs without [labels.*] continue to load unchanged (backward compatible)"
tags: ["config", "kind:enhancement", "triage:ready-for-agent"]
---

# [labels.*] config: parse single-select + closed-value namespaces

## What to build

Teach tkt about **label namespaces** declared in `.tickets/config.toml`, so a tag
prefix can be a validated, closed-value, optionally single-select axis (the
native replacement for emulating GitLab scoped labels with a tag convention).

```toml
[labels.triage]
single_select = true
values = ["needs-triage", "needs-info", "ready-for-agent", "ready-for-human", "wontfix"]
```

- `src/config.rs`: parse `[labels.<ns>]` tables into a namespace -> {single_select,
  values} map. Validation of tags against this lives in a sibling ticket (198).

Additive: configs without `[labels.*]` load unchanged.

## Acceptance criteria

- [ ] `config.rs` parses `[labels.<ns>]` tables with a `single_select` bool and a `values` list
- [ ] Unknown or malformed `[labels.*]` entries produce a clear config error
- [ ] Existing configs without `[labels.*]` continue to load unchanged (backward compatible)

## Triage — 2026-11-02

Disposition: `kind:enhancement`, `triage:ready-for-agent`; stream `config`.

Label namespace parsing is not present in the known config contract. Parse single_select/values at the boundary; #198 owns enforcement and must not be duplicated here.

This assessment supersedes conflicting historical planning notes above. Evidence, remaining unknowns and board coverage: [full-board triage](../docs/agents/ticket-triage-2026-11-02.md). Historical `.scratch` references are optional provenance, not required implementation inputs; use the durable inline contract and current tracked sources.
