---
id: "262"
title: "Separate native Claude validation from mandatory tooling fixtures"
status: open
blocked_by: []
validation_criteria:
  - "Routine tooling gates retain static and preservation checks without installed Claude; native fixture runs on explicit request."
tags: ["tooling"]
---
Intent: mandatory-gate review in grey-matter ticket 252 found the same host dependency in this pack. tools/release-tools.test.mjs imports tools/deploy-validation.test.mjs, including an installed Claude validator scenario. Split native integration into an explicitly on-demand suite while preserving cheap hook/config safety and static manifest checks. Current native scenario passed on macOS during the deployer vendor refresh; no failure is claimed. Acceptance: routine tooling gates do not depend on installed Claude; safety fixtures remain blocking; optional native suite remains runnable.