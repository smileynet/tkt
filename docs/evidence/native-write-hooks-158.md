# Native write-hook evidence — 2026-11-03

Independent critic inspection observed Codex CLI 0.166.0 (`hooks stable true`) and
Claude 2.1.316. These are tested local profiles, not established minimum versions.
No hook, trust record or user setting was changed by this investigation.

Codex currently supports PreToolUse for canonical apply_patch input at
`tool_input.command`, matcher aliases Edit/Write, and nonblocking model-visible
additionalContext. Project hooks need trusted project configuration plus review
of the exact definition; changes invalidate its hash-based trust. Tool coverage
has exceptions. Source: [official hooks reference](https://learn.chatgpt.com/docs/hooks).

Claude supports Edit|Write matching and PreToolUse additionalContext. Interactive
workspace trust differs from print/SDK operation, which treats the working folder
as trusted; command hooks have user permissions. Source:
[official Claude hooks reference](https://code.claude.com/docs/en/hooks).

The old support-absence gate is satisfied. Ticket 158 can implement explicit,
owned native config generation and bounded advisory path/patch detection. It must
not add deny decisions, permission policies, implicit activation/trust, or claims
of arbitrary shell/MCP write coverage. Git pre-commit installer 157 is a separate
feature, not the previously imagined shared native-hook configuration dependency.
Exact historical minimum versions remain unknown; do not invent a version gate.
Live native hook execution remains a distinct verification obligation/limit.
