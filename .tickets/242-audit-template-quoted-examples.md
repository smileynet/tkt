---
id: "242"
title: "Audit completed templates by actual section content"
status: done
blocked_by: []
priority: medium
validation_criteria:
  - "Quoted examples are ignored while actual unfinished templates and historical evidence warnings remain visible"
  - "Independent critic and full local gate pass; final corpus findings have evidence-backed dispositions"
tags: ["quality", "kind:bug", "triage:ready-for-agent", "stream:quality"]
assignee: "codex"
---
# Ignore quoted template examples in completed-ticket audits

## Intent source

Final unblocked-queue deep audit on 2026-11-03 and the deferred rule-change recommendation in docs/evidence/historical-closure-239.md. Installed audit source 768d392 reports template-only-closure on completed 176 because its substantive specification quotes the old placeholder string. Completed 149, 152 and 174 also discuss placeholder examples. Preserve historical prose and missing evidence.

## What to build

Make the done-ticket template rule inspect actual template section content rather than substrings anywhere in the body. Reuse the content-based stub logic used by validate where practical. Ignore quoted inline/fenced examples and nonmatching headings. Keep real empty/legacy/current prompt-only sections detectable and preserve migration/supersession exemptions. This is a mechanical rule correction, not a waiver of unknown historical evidence.

## Acceptance criteria

- [x] A public audit regression reproduces quoted-example false warnings before the fix and removes them after it.
- [x] Real unfinished intent or acceptance sections remain flagged, including the current default prompts; migration/supersession exclusions and status scope remain intact.
- [x] Open-ticket stub detection remains consistent, with tests for quoted/fenced examples and actual sections.
- [x] Independent critic accepts final source; full local gate passes and final deep audit findings are classified honestly.

## Current evidence

Scope clarification: in completed tickets, a checked placeholder still lacks
substantive content. Preserve warnings for real placeholder lines mixed with
substantive ACs in an actual acceptance section. The open/in-progress stub rule
may keep its existing sole-placeholder policy; share Markdown-aware section
parsing without silently broadening readiness policy.

Final deep audit after six CLI closures reports 15 findings: 14 warnings and one high-priority-open info. The 176 template warning is a confirmed false positive: its actual acceptance criteria are checked and its body includes a quoted example of an old unchecked placeholder. Five historical missing-evidence warnings remain unknown, not waived. See docs/evidence/frontier-maintenance-2026-11-03.md for the preceding verified queue.

## Resolution (2026-11-03)

Completed-template audit reads actual sections and ignores quoted examples, using a shared bounded section parser and explicit lifecycle policies. Public boundary regressions, independent review, full gate and installed corpus checks passed. Historical prose and missing proof remain intact; evidence promoted and temporary delivery artifacts expire after snapshot publication.

### Verification
1. ✓ Quoted examples are ignored while actual unfinished templates and historical evidence warnings remain visible — "Public CLI regression red before, green after; shared section reader covers quoted/fenced, real empty/current/legacy, checked/mixed and repeated sections while preserving lifecycle policies. Installed audit removes four false positives and retains historical evidence warnings. See docs/evidence/audit-template-242.md."
2. ✓ Independent critic and full local gate pass; final corpus findings have evidence-backed dispositions — "Independent critic accepted frozen source; full gate passed 385 Rust tests and 42 Node tests with one existing Windows skip, formatting and Clippy clean. Installed source 5f4d3f4 strict corpus and plan checks passed; advisory audit retains 11 classified findings. Durable source receipt promoted, shared cache retained for maintenance and delivery helpers expire at final dashboard update."
