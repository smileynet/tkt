---
id: "260"
title: "Route remaining filesystem mutation families through typed effect phases"
status: in_progress
blocked_by: []
priority: medium
validation_criteria:
  - "Prepared local phases distinguish create, snapshot replacement, owned removal and rename with safe preview preparation"
  - "Remaining ticket, migration, renumber, projection, hook and journal writes use the shared executor without losing ownership or recovery behavior"
tags: ["architecture", "integrity", "kind:enhancement", "triage:ready-for-agent", "stream:quality"]
parent: "191"
---
Implementation slice of parent191. Keep transaction owners distinct and use concrete effect data, not callbacks. Preserve existing output/error contracts, atomic writes, exact ownership and quarantine recovery. Account for marker/lock/diagnostic/cache effects; preview must not persist any. Parent191 remains open for Git/process phases and the full matrix. Independent critic review before closure.