---
id: "04"
title: "cycle detection in validate command"
status: open
blocked_by: []
---

# Cycle detection in validate command

## What to build

The README claims `validate` checks for cycles in the dependency graph, but it currently doesn't. Python tkt has `_cycles` with DFS-based cycle detection. Add this to the Rust `validate` command.

### Algorithm

1. Build adjacency map: `id → Vec<blocked_by_ids>` (after duplicate/dangling checks)
2. DFS with three states: unvisited, visiting (in current path), visited (complete)
3. When a "visiting" node is reached, extract the cycle path
4. Report each unique cycle exactly once, deterministically sorted
5. Classify as error (not warning)

### Output format

```json
{"file": "04-feature.md", "rule": "cycle", "severity": "error", "message": "dependency cycle: 04 → 02 → 04"}
```

### Edge cases

- Self-cycle: `blocked_by: ["04"]` in ticket 04
- Two-node cycle: 04 blocks 02, 02 blocks 04
- Longer chains: 01 → 02 → 03 → 01
- Multiple disjoint cycles in same corpus
- Cycle adjacent to dangling reference (both reported independently)
- Node in cycle that also has valid non-cyclic deps

## Acceptance criteria

- [ ] Self-cycle detected and reported
- [ ] Two-node cycle detected and reported
- [ ] 3+ node cycle detected with full path
- [ ] Multiple disjoint cycles each reported independently
- [ ] Each cycle reported exactly once (no duplicates)
- [ ] Cycle path is deterministic (sorted start node)
- [ ] Cycles classified as errors (affect exit code with --strict)
- [ ] Integration test with cyclic corpus
- [ ] Integration test with acyclic corpus (no false positives)
- [ ] Does not interfere with existing dangling-ref detection
