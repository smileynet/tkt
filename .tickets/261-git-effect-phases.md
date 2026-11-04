---
id: "261"
title: "Route Git publication and process phases through the common effect dispatcher"
status: done
blocked_by: ["260"]
priority: medium
validation_criteria:
  - "Typed Git dispositions delegate to existing transaction owners without opaque callbacks or merging their algorithms"
  - "The complete mutator matrix verifies effect previews and zero filesystem, Git or process changes including recovery and remote-dependent cases"
tags: ["architecture", "integrity", "kind:enhancement", "triage:ready-for-agent", "stream:quality"]
parent: "191"
---
Final implementation slice of parent191 after260. Phase-local plans may re-materialize after fetch/CAS/retry; never replay stale snapshots. Include allocation, existing-ticket publish, undo, rebase, lifecycle process events and persistent diagnostics. Remote-dependent previews must label unresolved effects. Integrate the pending mutation_preview.rs suite and private telemetry controls. Independent critic review before closure; parent191 closes explicitly only when every original criterion is evidenced.

## Resolution (2026-11-04)

Completed typed Git/process/support dispatcher and real prepared preview routing; preserved scoped Git ownership, atomic publication and push.enabled. Critic accepted261 and parent191 after final gate. Installed e755765 and check-binary-fresh passed. Promoted compact receipt docs/evidence/git-effects-261.md; recycled57 owned intermediate logs/scripts. Shared target/dist retained for maintainer builds/deployment; final gate/install logs retained through parent191 closure, then recycle. No hosted CI or release.

### Verification
1. ✓ Typed Git dispositions delegate to existing transaction owners without opaque callbacks or merging their algorithms — "Typed effects::Phase delegates Git publication, allocation, import recovery, undo, rebase and lifecycle/support phases to distinct owners; source e755765 and independent critic acceptance; docs/evidence/git-effects-261.md."
2. ✓ The complete mutator matrix verifies effect previews and zero filesystem, Git or process changes including recovery and remote-dependent cases — "Fresh mise run check:523 Rust/42 Node pass,0failures; exact29-route matrix, cached/stale remote, partial clone, import recovery and private telemetry controls; documented helper ignore/Windows skip."
