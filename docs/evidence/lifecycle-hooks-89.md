# Advisory lifecycle hook evidence

PASS: fresh `mise run check` on final source completed 496 Rust tests and 42 Node
checks, clean `cargo fmt --check` and warning-free all-target clippy. SKIP: one
existing Windows relative-link fixture. The lifecycle harness passed 14 cases
(13 behavioral cases plus a private child entry) in 6.26 seconds.

The public CLI fixtures cover all three event identities and committed revisions,
JSON stdout preservation, every lifecycle preview, no configuration, failed
commits, absent/false/nonliteral/duplicate/whitespace local trust, project/global/
included self-authorization refusal, linked-worktree trust, spawn failure,
nonzero exit, deadline expiry, recursive calls, publication before invocation and
retained commits after failed publication. A real Git allocation race verifies
only the final ticket emits an event. Deterministic native Git probes insert a
concurrent commit before revision capture and another after successful push;
neither unrelated revision becomes an event. Existing importer recovery tests
also pass with the extended private probe.

Critic review required raw exact trust bytes rather than trimmed Git output,
post-publication identity checks, and a pre-commit parent snapshot around capture.
Those changes are present in the final passing gate. The independent critic
accepted the final source, guidance and fresh gate before ticket closure. Earlier intermediate gates
are not final evidence: one predates the attribution fixes, and another failed
because the extended private probe moved values before its new post-push action.
Borrowing those values fixed the probe; the final full gate passed.

PASS: pack lint, native Antigravity strict validation (no warnings), and guidance
coverage (45 units/355 frozen source lines). Full hook semantics live in the
standalone command reference with short mandatory core/work pointers; all init
snippets, capabilities, README and contributor contract reflect opt-in trust and
advisory behavior. Source identities are in the companion JSON.

See [configuration, trust and recovery](../lifecycle-hooks.md). Hooks execute only
after a committed publication attempt and cannot gate it. Trusted programs can
have arbitrary external effects; the CLI never rolls back their effects. The
two-second bound terminates/reaps the direct child only. No descendant
containment, durable outbox, authentication or automatic replay is claimed.
Windows execution was observed; Unix native execution remains unverified. No
lifecycle hooks or local trust were enabled in this repository during delivery.

Owned scratch logs are promoted into this compact receipt and recycled. Shared
build outputs and managed plugin caches remain for ongoing tickets; unrelated
artifacts remain untouched. Parent #191's all-mutator consolidation is unfinished.
