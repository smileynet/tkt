---
id: "288"
title: "publish a GitHub-source marketplace so both dev machines converge via plugins update"
status: backlog
blocked_by: []
priority: low
validation_criteria:
  - "a published marketplace source (GitHub, or archive pinned by sha256 to the release bundle) is the canonical install; local-directory adds remain a dev-machine affordance"
  - "macOS (0.1.0 spike snapshot) and Windows (0.2.0) installs converge through plugins update from the published source"
tags: ["kind:enhancement", "stream:tooling"]
---
From grey-matter proposal 23 (T4), pairs with grey-matter ticket 292 (packed-marketplace-archive-source). Anthropic's official marketplace uses zero dev-checkout sources; archive sources from release assets are the documented lightweight route.