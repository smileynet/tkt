---
id: "260"
title: "Route remaining filesystem mutation families through typed effect phases"
status: done
blocked_by: []
priority: medium
validation_criteria:
  - "Prepared local phases distinguish create, snapshot replacement, owned removal and rename with safe preview preparation"
  - "Remaining ticket, migration, renumber, projection, hook and journal writes use the shared executor without losing ownership or recovery behavior"
tags: ["architecture", "integrity", "kind:enhancement", "triage:ready-for-agent", "stream:quality"]
parent: "191"
---
Implementation slice of parent191. Keep transaction owners distinct and use concrete effect data, not callbacks. Preserve existing output/error contracts, atomic writes, exact ownership and quarantine recovery. Account for marker/lock/diagnostic/cache effects; preview must not persist any. Parent191 remains open for Git/process phases and the full matrix. Independent critic review before closure.

## Note — 2026-11-04

By: anonymous

<!-- tkt-note:v1 104428-1793784865791794700 -->
Execution boundary: this slice centralizes command-owned text publication/removal (including TicketFile, migration, renumber, projection, hook files and import journals). Ticket261 explicitly owns remaining Git/process phases and their ephemeral support edges: allocation/publisher locks, undo temporary index, cadence state, telemetry/debug rotation and update cache. Those are inventoried as remaining work, not claimed centralized by260. Parent191 stays in_progress until261 and the original all-mutator matrix are evidenced.

## Resolution (2026-11-04)

Implemented typed prepared filesystem phases in79bd245 and deployed at5503c4d. All command-owned text writes now route through the dispatcher; original ticket snapshots reject stale writes, relocation is no-clobber and owned removal preserves foreign arrivals. Context clear retains symlink unlink behavior. Fresh gate516Rust/42Node,fmtclean/clippy0; explicit helper/direct-run exclusion and existing Windows skip recorded. Critic accepted before closure. Git/process/support effects remain261; parent191 remains open. See docs/evidence/filesystem-effects-260.md.

### Verification
1. ✓ Prepared local phases distinguish create, snapshot replacement, owned removal and rename with safe preview preparation — "Explicit CreateNoClobber,ReplaceSnapshot,RelocateSnapshot,RemoveOwned,UnlinkEntry and directory variants; safe preparation in Preview plus Apply rechecks. Focused stale-source/late-target/relocation controls pass."
2. ✓ Remaining ticket, migration, renumber, projection, hook and journal writes use the shared executor without losing ownership or recovery behavior — "TicketFile,new,batch,lint,plan,migration,renumber,hook,journal and native config text operations use the executor; full gate and existing recovery fixtures pass. Independent critic accepted final source and compatibility repair."
