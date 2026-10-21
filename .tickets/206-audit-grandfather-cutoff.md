---
id: "206"
title: "audit: grandfather pre-protocol corpus from missing-resolution (ADR 0003)"
status: in_progress
blocked_by: []
priority: medium
validation_criteria:
  - "config [audit] cutoff key parsed; resolution-quality rules exempt done tickets below cutoff"
  - "low-evidence-closure and AC-completeness rules are NOT exempted by the cutoff"
  - "regression test: a post-cutoff done ticket with no Resolution still fires"
tags: ["config"]
---

# audit: grandfather pre-protocol corpus from missing-resolution (ADR 0003)

## What to build

Implement the audit-debt baseline decided in ADR 0003
(`.memory/adr/0003-audit-debt-grandfather-cutoff.md`). `tkt audit --strict` currently
reports 27 `missing-resolution` findings on this repo's own tickets 02–29 and 35 —
done tickets closed before the Resolution-section convention existed, whose evidence
cannot be honestly reconstructed. The policy is to grandfather that pre-protocol
corpus rather than fabricate retrospective resolutions.

Add a configurable cutoff so the resolution-quality rules
(`check_resolution_quality` in `src/audit.rs:12`, rules `missing-resolution` and
`tbd-resolution`) exempt done tickets below the cutoff.

- `src/config.rs`: parse a `[audit]` section with a cutoff key (e.g.
  `resolution_baseline_id` or a closed-before date). Default it to the known
  pre-protocol boundary for this repo so `audit --strict` drops to the 3
  `low-evidence-closure` findings out of the box.
- `src/audit.rs`: thread the cutoff into `check_resolution_quality` so tickets below it
  are skipped for the two resolution-quality rules only.
- The cutoff MUST NOT exempt `low-evidence-closure` or AC-completeness rules.

Honest baseline, not fabrication: we never invent evidence for work whose evidence is
gone — we record that the pre-protocol corpus is exempt.

## Acceptance criteria

- [ ] A `[audit]` config key sets the cutoff; resolution-quality rules exempt done
      tickets below it (default drops this repo's 27 pre-protocol findings)
- [ ] `low-evidence-closure` and AC-completeness rules are NOT affected by the cutoff
- [ ] Regression test: a post-cutoff done ticket with no `## Resolution` still fires
      `missing-resolution`
- [ ] Unknown/malformed `[audit]` config produces a clear config error (consistent with
      the #190 config-parsing direction)
