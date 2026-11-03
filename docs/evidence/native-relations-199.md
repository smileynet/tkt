# Combined native relations walkthrough

The capability manifest now describes optional parent/assignee semantics and
label namespace configuration. All six generated instruction targets include
these rules; the combined guide is `docs/native-relations.md`, linked from README.
The changelog records computed progress and additive compatibility, without
claiming automatic parent closure or weakened validation.

- Pass: final `mise run check`: 396 Rust tests; 42 Node fixtures pass and one
  existing Windows relative-link fixture skipped. Formatting and warning-denied
  clippy pass. All six parent integration tests pass.
- Pass: the combined end-to-end fixture creates a child under an epic, verifies
  container frontier exclusion, claims under an invocation identity, queries its
  assignee, explicitly closes with AC/evidence, and reads parent progress without
  changing parent status. It exercises single-select and closed-value failures,
  restores valid labels, then passes strict validation.
- Pass: manifest discovery and the actual written files for agents, Claude,
  Copilot, Cursor, Kiro and Windsurf contain all three native concepts.
  An initial test incorrectly inspected target-command stdout as snippet content;
  it failed, and was corrected to inspect generated temporary-repository files.
  A local probe's generated contributor snippet was removed before scope review.
- Pass: deployment precheck and scoped `git diff --check`.
- Independent critic accepted the combined final implementation and guide:
  capabilities.rs AF54B9A79F157F28F075114F4FE0D04D56AE01F98BF444CA1CA205EC128A349B;
  init.rs 5F1FFECB57967B1EFEE56B9DA58998B9E8693C6644BE3D73105AF01F261907E3;
  parent_cli.rs 6A2FD49FFEACBCC9367BC30378E3C49E917DA9DF85A5B33B6F530A330C63EC8F;
  native-relations.md AAA44129DA5085951BFDD6B53C7DE68CA0DBA81123BD5351024EB0468062418C.

Proof is promoted here; disposable test/gate logs are released to OS trash.
The shared target cache is retained for subsequent CLI work. No UI rendering or
user outcome is established by these tests. Existing tickets remain valid when
the new optional fields/configuration are absent; declared policy is still checked.
