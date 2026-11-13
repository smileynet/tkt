---
id: "286"
title: "SessionStart hook fails under ZCode: command/args manifest mix and non-JSON stdout"
status: done
blocked_by: []
validation_criteria:
  - "hook manifest invokes the script under ZCode (process-type or single command string)"
  - "hook stdout satisfies ZCode strict hookEventName JSON while preserving Claude Code compatibility"
  - "post-fix ZCode log shows outcome success and frontier context injected on a fresh session start"
tags: ["tooling"]
---
docs/evidence/zcode-plugin-live-validation-282.md records the finding: .claude-plugin/claude-hooks.json declares type:command with an args array (a process-type field), so ZCode spawns bare node, which exits 1 on the hook input (logged hook.run.failed, 414 ms, five session starts incl. 2026-11-08). Independently, the hook prints plain text, which ZCode's strict hookEventName/additionalContext JSON schema rejects, so output would be discarded even with a correct invocation. Fix the manifest (type:process with command/args/timeoutMs, or one command string) and the output shape ({hookEventName:SessionStart,additionalContext:...}); verify Claude Code compatibility before changing output. Acceptance: ZCode CLI log shows the hook succeeding and context injected on a new session start; standalone run still exits 0.

## Resolution (2026-11-13)

Fixed the tkt plugin SessionStart hook under ZCode: hooks manifest uses a single shell-string command (no Claude-only args mix) surviving GUI hosts' minimal PATH, and the script emits strict hookEventName JSON. Verified live on a fresh ZCode session 2026-11-13: hook executed (851ms phase, completed), frontier context injected, zero hook.run.failed.

### Verification
1. ✓ hook manifest invokes the script under ZCode (process-type or single command string) — "Live fresh-session run under ZCode Desktop 2026-11-13: session sess_ac71ac6a turn_f7b8b328 executed the plugin SessionStart hook via the single shell-string command form (manifest byte-identical to HEAD b23414a in ~/.zcode/cli/plugins/cache/tkt/tkt/0.4.0); session_start_hooks phase completed in 851ms under the GUI host's minimal PATH, exercising the b23414a PATH fix"
2. ✓ hook stdout satisfies ZCode strict hookEventName JSON while preserving Claude Code compatibility — "ZCode injected the hook's additionalContext (tkt frontier 287/286 + in-progress block), which only happens for strict hookEventName-discriminated JSON stdout; Claude Code compatibility unchanged (claude-hooks.json single command string, no args field; gate green at fix time, mise run check + 533 tests)"
3. ✓ post-fix ZCode log shows outcome success and frontier context injected on a fresh session start — "zcode-2026-11-13.jsonl: zero hook.run.failed events; first-turn session_start_hooks phase status=completed (851ms vs ~1ms resume no-op) with frontier context injected into the fresh session; details in docs/evidence/zcode-session-spawn-286.md (df13e9b)"
