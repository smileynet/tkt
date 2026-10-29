# Workspace Layout

```
src/
├── main.rs          — entry point + ErrorKind enum + DomainError struct
├── cli.rs           — clap derive commands + dispatch + JSON envelope helpers
├── color.rs         — color/symbol support (NO_COLOR, --color, TKT_ASCII)
├── config.rs        — unified config cascade (user + project + env)
├── context.rs       — context state (load/save .tickets/.context)
├── telemetry.rs     — consent, session tracking, JSONL sink, rotation
├── migrate.rs       — schema detection + conversion (tk → tkt)
├── mutation.rs      — MutationContext (push-gated lifecycle for existing-ticket mutations)
├── renumber.rs      — RenumberPlan (pure ID remapping: plan + apply)
├── audit.rs         — pure audit rules (injectable deps, no I/O)
├── core/
│   ├── mod.rs       — re-exports
│   ├── ticket.rs    — TicketFile (raw editor + typed mutations) + Ticket (typed domain)
│   └── validate.rs  — input validation (slugs, free text, IDs, enums)
├── commands/        — one file per subcommand (new, close, claim, edit, lint, doctor, etc.)
├── findings.rs      — validation rules, Finding struct, output formatting
├── transaction.rs   — GitTransaction (allocation: fetch→scan→commit→push→retry)
└── git.rs           — git subprocess wrapper (fetch, commit, push, remote scanning)
tests/
├── integration.rs   — integration tests (tempdir repos, race scenarios, telemetry, debug)
└── parity/          — historical Python parity comparison harness
skills/                — cross-harness skills (deployed as copy to ~/.{kiro,codex,claude,agents}/skills/ and packaged into ~/.gemini/config/plugins/tkt/)
  tkt-core/            — agentskills.io skill: tkt CLI jobs-to-be-done + references
  tkt-orient/          — session-start: read reconciled plan, propose next pick
  tkt-work/            — execution loop: claim, verify premises, do work, close with evidence
  tkt-reconcile/       — session-end: make tickets current, re-sequence the plan
plugin.json          — Antigravity (agy) / Agent Plugins manifest (packaged with skills/ by tools/deploy-skills.sh)
GLOSSARY.md   — project glossary
.tickets/            — tkt's own tickets (dogfooding)
.references/         — cloned reference repos (gitignored)
TELEMETRY.md         — transparency document for telemetry collection
```

See the [reference inventory and clone commands](../../../.references/README.md).
