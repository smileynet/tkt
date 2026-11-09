---
id: "286"
title: "SessionStart hook fails under ZCode: command/args manifest mix and non-JSON stdout"
status: open
blocked_by: []
validation_criteria:
  - "hook manifest invokes the script under ZCode (process-type or single command string)"
  - "hook stdout satisfies ZCode strict hookEventName JSON while preserving Claude Code compatibility"
  - "post-fix ZCode log shows outcome success and frontier context injected on a fresh session start"
tags: ["tooling"]
---
docs/evidence/zcode-plugin-live-validation-282.md records the finding: .claude-plugin/claude-hooks.json declares type:command with an args array (a process-type field), so ZCode spawns bare node, which exits 1 on the hook input (logged hook.run.failed, 414 ms, five session starts incl. 2026-11-08). Independently, the hook prints plain text, which ZCode's strict hookEventName/additionalContext JSON schema rejects, so output would be discarded even with a correct invocation. Fix the manifest (type:process with command/args/timeoutMs, or one command string) and the output shape ({hookEventName:SessionStart,additionalContext:...}); verify Claude Code compatibility before changing output. Acceptance: ZCode CLI log shows the hook succeeding and context injected on a new session start; standalone run still exits 0.