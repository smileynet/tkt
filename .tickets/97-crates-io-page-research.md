---
id: "97"
title: "Research top crates.io projects for description page style and best practices"
status: done
blocked_by: []
priority: medium
validation_criteria: 
  - "Research covers 5+ top crates"
  - "Findings written to .scratch/research/"
  - "Specific recommendations for tkt"
  - "Recommended changes applied"
---

# Research top crates.io projects for description page style and best practices

## What to build

Dispatch subagents to research how the top CLI crates on crates.io present themselves — specifically what shows up on the crates.io page (not just the README). Consider:

- How top crates use the description field (length, tone, keywords)
- Whether they set `documentation` to a custom URL vs letting docs.rs auto-link
- How `homepage` is used (dedicated site vs GitHub repo)
- What the crates.io rendered page looks like for tools with good discoverability
- Whether any use `[package.metadata]` for additional presentation
- The relationship between GitHub stars, downloads, and page presentation

Research targets: ripgrep, bat, fd, just, delta, starship, tokio, serde, clap — mix of CLI tools and popular libraries.

Deliverable: recommendations for tkt's crates.io presence (description wording, fields to add/change, any actions to take before v0.2.0).

## Acceptance criteria

- [x] Research dispatched covering 5+ top crates
- [x] Findings written to .scratch/research/
- [x] Specific recommendations for tkt's Cargo.toml and crates.io page
- [x] Any recommended changes applied before v0.2.0

# Research top crates.io projects for description page style and best practices

## What to build

TBD

## Acceptance criteria

- [ ] TBD

## Resolution (2026-08-12)

Researched 6 top crates. Applied: homepage, documentation link, replaced 'cli' keyword with 'todo'.

### Verification
1. ✓ Research covers 5+ top crates — "Researched ripgrep, bat, fd-find, just, starship, delta (6 crates)"
2. ✓ Findings written to .scratch/research/ — "Written to .scratch/research/crates-io-pages.md"
3. ✓ Specific recommendations for tkt — "Recommendations: add homepage, docs link, fix keywords"
4. ✓ Recommended changes applied — "Cargo.toml updated with homepage, documentation, keywords"
