---
id: "259"
title: "Prevent telemetry commands from mutating during dry-run"
status: in_progress
blocked_by: []
priority: high
validation_criteria:
  - "Dry-run telemetry enable, disable and clear preserve consent and data without accessing ambient user state in tests"
  - "Private path controls prove real apply still writes consent and clears only the selected telemetry directory"
tags: ["integrity", "kind:bug", "triage:ready-for-agent", "stream:quality"]
parent: "191"
---
The all-mutator matrix on Windows discovered that dirs::config_dir/data_dir ignored redirected APPDATA/XDG variables. Global dry-run telemetry --enable/--disable wrote the ambient consent file and --clear ran ambient deletion. The real consent is now disabled; no prior snapshot exists, so prior contents and removed data are unknown. Stop explicit telemetry integration cases until the preview boundary is fixed. Add typed, testable prepared telemetry effects and private-path unit controls; never test consent mutations against ambient paths. Independent critic review required before closure. Parent191 still owns complete executor consolidation and all-mutator matrix.