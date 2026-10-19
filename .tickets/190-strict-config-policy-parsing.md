---
id: "190"
title: "Parse known config keys into strict typed policies"
status: backlog
blocked_by: []
priority: high
validation_criteria:
  - "Invalid known config values fail with structured field context"
  - "Evidence, push, priority, and environment policies use typed parsed values"
  - "A typo cannot silently disable required evidence or publishing"
  - "Configuration compatibility cases are covered by tests"
tags: ["config", "integrity"]
---

# Parse known config keys into strict typed policies

## What to build

Known project and user configuration keys are read as raw strings in several policy paths.
Typos such as `push.enabled = "treu"` can currently collapse to false, while an unrecognized
evidence value can disable a required gate. The configuration boundary must either produce a
valid typed domain value or return a structured, actionable error; it must not silently choose
a permissive policy.

Parse booleans, evidence policy, priority, environment, and other closed-domain keys into
typed values once while retaining source/key context for diagnostics. Surface validation through
the ordinary configuration/command error path, including merged user/project settings. Keep
#139 focused on evidence-gate semantics and #165 on safely writing project-scoped config; this
ticket owns safe reading and interpretation.

## Acceptance criteria

- [ ] Invalid known config values fail with structured key, value, and source context
- [ ] Evidence, push, priority, and environment policies consume typed parsed values
- [ ] A typo cannot silently disable required evidence, publishing, or a filter
- [ ] Unknown-key compatibility behavior is documented and tested separately from known keys
- [ ] User/project precedence and valid legacy values have regression coverage
