---
id: "43"
title: "investigate mutation command latency: 2-2.4s average from git push"
status: open
blocked_by: []
---

# Investigate mutation command latency: 2-2.4s average from git push

## Observed

Telemetry shows mutation commands averaging 2+ seconds:
- `new`: 2379ms (n=53)
- `close`: 2268ms (n=33)
- `edit`: 1939ms (n=8)
- `claim`: 1876ms (n=2)

Read commands are fast (50-96ms). The difference is entirely git push + fetch overhead.

## Breakdown (estimated)

```
git fetch        ~300-500ms (network)
load corpus      ~10-50ms (local I/O)
write file       ~1ms
git add+commit   ~50-100ms (local)
git push         ~1000-1500ms (network)
─────────────────────────────
Total            ~1500-2400ms
```

## Options to investigate

1. **Defer push** — batch mutations and push once at end of session (risky: loses push-to-claim atomicity)
2. **Background push** — push in a fire-and-forget subprocess, return immediately (fast but doesn't confirm success)
3. **Skip fetch for known-fresh repos** — if we pushed <N seconds ago, skip the fetch on next mutation (stale check is redundant immediately after our own push)
4. **Accept it** — 2s is the cost of atomic remote operations. Document as expected behavior.
5. **Offer --no-push flag** — for local-first workflows where push happens separately (batch at session end)

## Recommendation

Option 4 (accept) for v1, with option 5 as a future enhancement for high-throughput sessions (like lacrosse-bosse-helper closing 5 tickets in 30 seconds). The latency is inherent to the push-to-claim design — it's the feature, not a bug.

## Acceptance criteria

- [ ] Decision documented
- [ ] If implementing --no-push: design + implementation
- [ ] If accepting: document expected latency in README/TELEMETRY.md
