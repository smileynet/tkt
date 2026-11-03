---
id: "242"
title: "Audit completed templates by actual section content"
status: open
blocked_by: []
priority: medium
validation_criteria:
  - "Quoted examples are ignored while actual unfinished templates and historical evidence warnings remain visible"
  - "Independent critic and full local gate pass; final corpus findings have evidence-backed dispositions"
tags: ["quality", "kind:bug", "triage:ready-for-agent", "stream:quality"]
---
# Ignore quoted template examples in completed-ticket audits

## Intent source

Final unblocked-queue deep audit on 2026-11-03 and the deferred rule-change recommendation in docs/evidence/historical-closure-239.md. Installed audit source 768d392 reports template-only-closure on completed 176 because its substantive specification quotes the old placeholder string. Completed 149, 152 and 174 also discuss placeholder examples. Preserve historical prose and missing evidence.

## What to build

Make the done-ticket template rule inspect actual template section content rather than substrings anywhere in the body. Reuse the content-based stub logic used by validate where practical. Ignore quoted inline/fenced examples and nonmatching headings. Keep real empty/legacy/current prompt-only sections detectable and preserve migration/supersession exemptions. This is a mechanical rule correction, not a waiver of unknown historical evidence.

## Acceptance criteria

- [ ] A public audit regression reproduces quoted-example false warnings before the fix and removes them after it.
- [ ] Real unfinished intent or acceptance sections remain flagged, including the current default prompts; migration/supersession exclusions and status scope remain intact.
- [ ] Open-ticket stub detection remains consistent, with tests for quoted/fenced examples and actual sections.
- [ ] Independent critic accepts final source; full local gate passes and final deep audit findings are classified honestly.

## Current evidence

Final deep audit after six CLI closures reports 15 findings: 14 warnings and one high-priority-open info. The 176 template warning is a confirmed false positive: its actual acceptance criteria are checked and its body includes a quoted example of an old unchecked placeholder. Five historical missing-evidence warnings remain unknown, not waived. See docs/evidence/frontier-maintenance-2026-11-03.md for the preceding verified queue.
