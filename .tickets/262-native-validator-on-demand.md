---
id: "262"
title: "Separate native Claude validation from mandatory tooling fixtures"
status: done
blocked_by: []
validation_criteria:
  - "Routine tooling gates retain static and preservation checks without installed Claude; native fixture runs on explicit request."
tags: ["tooling"]
---
Intent: mandatory-gate review in grey-matter ticket 252 found the same host dependency in this pack. tools/release-tools.test.mjs imports tools/deploy-validation.test.mjs, including an installed Claude validator scenario. Split native integration into an explicitly on-demand suite while preserving cheap hook/config safety and static manifest checks. Current native scenario passed on macOS during the deployer vendor refresh; no failure is claimed. Acceptance: routine tooling gates do not depend on installed Claude; safety fixtures remain blocking; optional native suite remains runnable.

## Resolution (2026-11-04)

done: moved installed Claude compatibility to an explicitly on-demand native suite, retained blocking static manifest/byte-preservation fixtures and existing hook/migration safety. Updated maintenance guidance and dashboard; committed, pushed and redeployed. Disposable fixtures auto-cleaned; task helper scripts moved to Trash; pre-existing dist496KiB preserved.

### Verification
1. ✓ Routine tooling gates retain static and preservation checks without installed Claude; native fixture runs on explicit request. — "node --test tools/release-tools.test.mjs tools/release-ci.test.mjs tools/check-binary-fresh.test.mjs:43/43 passed, no skips,13.5s, including static validation and existing hook/migration/publication safety. Focused static/hook/migration suite12/12 passed. Explicit on-demand tools/deploy-native.test.mjs:1/1 passed,1.7s. Static and native deployment checks passed. git diff --check and dashboard checker passed without findings. Commit3716c08 pushed. node tools/deploy.mjs completed: Claude source verified, Codex/Kiro/OpenCode/Antigravity current, strict AGY validation passed. Tooling-only scope; no full Rust or other-platform matrix claimed."
