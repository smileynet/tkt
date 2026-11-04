---
id: "259"
title: "Prevent telemetry commands from mutating during dry-run"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "Dry-run telemetry enable, disable and clear preserve consent and data without accessing ambient user state in tests"
  - "Private path controls prove real apply still writes consent and clears only the selected telemetry directory"
tags: ["integrity", "kind:bug", "triage:ready-for-agent", "stream:quality"]
parent: "191"
---
The all-mutator matrix on Windows discovered that dirs::config_dir/data_dir ignored redirected APPDATA/XDG variables. Global dry-run telemetry --enable/--disable wrote the ambient consent file and --clear ran ambient deletion. The real consent is now disabled; no prior snapshot exists, so prior contents and removed data are unknown. Stop explicit telemetry integration cases until the preview boundary is fixed. Add typed, testable prepared telemetry effects and private-path unit controls; never test consent mutations against ambient paths. Independent critic review required before closure. Parent191 still owns complete executor consolidation and all-mutator matrix.

## Resolution (2026-11-04)

Fixed telemetry preview boundary in 0cfa35d and deployed the fresh CLI. Preview returns before platform path resolution; event recording and cleanup skip preview. Private unit matrix and apply controls plus linked-root refusal pass. Final full gate512Rust/42Node,one existing Windows skip,fmt clean,clippy0; critic accepted. The initial isolation mistake changed real consent to disabled and invoked clear; previous consent/data remain unknown, as recorded in docs/evidence/telemetry-preview-259.md.

### Verification
1. ✓ Dry-run telemetry enable, disable and clear preserve consent and data without accessing ambient user state in tests — "Private-path table covers enable/disable/clear, existing/absent files and a panic resolver proving preview cannot resolve platform user folders. Critic accepted final boundary."
2. ✓ Private path controls prove real apply still writes consent and clears only the selected telemetry directory — "Private apply controls verify atomic consent writes and exact selected data removal while preserving unrelated files. Linked-root removal is refused. Fresh gate and deployed CLI verified at0cfa35d."
