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
skills/tkt/          — agentskills.io skill (deployed via symlink to ~/.kiro/skills/tkt)
steering/            — always-on steering files (deployed as copy to ~/.kiro/steering/)
plugin.json          — Agent Plugins v1.0.0 manifest
.memory/CONTEXT.md   — project glossary
.tickets/            — tkt's own tickets (dogfooding)
.references/         — cloned reference repos (gitignored): clispec, axocli, octo-cli
TELEMETRY.md         — transparency document for telemetry collection
```
