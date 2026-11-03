# Architecture reference

Read when changing modules, ticket parsing, Git transactions, telemetry, errors,
terminal output, dependencies or facade exports. These are implementation
constraints; recorded domain decisions remain in [the ADRs](../adr/).

Prepared under ticket 245. Original text is duplicated during the bounded
transition; ticket 246 removes the extracted source from AGENTS.md.

<a id="rule-18"></a>

## Architecture Decisions

- **Shell out to git** (not libgit2): full SSH/HTTPS auth compat, matches gh CLI pattern, simplest v1
- **TicketFile + Ticket split**: TicketFile owns raw frontmatter for surgical edits; Ticket provides typed, validated fields (Status enum, zero-cost &str access). Mutations go through `.file`, reads use typed fields directly.
- **Custom frontmatter parser**: line-based key:value parsing with raw preservation. Supports YAML double-quoted scalar escaping (encode on write, decode on read). Not a full YAML parser — deliberately supports a narrow, round-trip-safe subset.
- **No async**: all operations are sequential (fetch → scan → write → commit → push)
- **Local-only telemetry**: opt-in JSONL file sink, per-project segmentation, session-aware rotation, never blocks CLI
- **LazyLock regex statics**: fixed patterns compiled once via `std::sync::LazyLock`
- **No color crate**: raw ANSI codes + `std::io::IsTerminal` — zero additional dependencies
- **Structured errors**: `DomainError { kind: ErrorKind, message, hint }` with 8-variant `ErrorKind` enum. `domain_bail!(Kind, "msg")` or `domain_bail!(Kind, "msg", hint: "fix")` or `domain_bail!("msg")` (defaults to Validation). JSON envelope emitted to stderr when `-o json`.
- **Facade re-exports**: only re-export types from `core/mod.rs` that callers name directly; types returned by methods but never named (e.g., `AcStats`) stay unexported from the facade to avoid unused-import warnings
