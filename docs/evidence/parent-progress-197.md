# Parent progress and explicit closure

Implemented `tkt children ID`: all direct children, independent of tag context,
with stored parent status, done/total and a distinct cancelled count. Total
includes every direct child, including cancelled; nested descendants are not
counted twice. JSON is one object; quiet text lists child IDs. Unknown IDs return
NotFound (exit 1). Reads do not mutate tickets. The frontier excludes tickets
referenced as a parent by any parsed child, preserving priority then ID sorting.
No close policy or stored lifecycle status is derived from child progress.

## Observed verification

- Pass: final `mise run check`: 42 Node fixtures passed, one existing Windows
  relative-link fixture skipped; `cargo fmt --check`, clippy with warnings denied,
  and every Rust test suite passed. The parent integration suite passed all five
  tests, including mixed states, full-corpus counts under unmatched tag context,
  empty/missing targets, nested containers, priority/ID tie-breaking, and a parent
  remaining open until a close with resolution/AC/evidence succeeds.
- Earlier attempts: one formatting assertion failure (module order), followed by
  a missing-target assertion failure (the old generic lookup produced exit 2).
  Both were fixed before the final full gate; neither is counted as a pass.
- Pass: native strict Claude manifest validation and `claude --plugin-dir . plugin
  details tkt` (four workflow skills); `node tools/deploy.mjs check` reports the
  plugin manifest check OK. No version bump/release was performed.
- Pass: scoped diff review and `git diff --check`.
- Independent critic accepted after correcting the earlier readiness rule and
  adding an ID tie-break fixture. Accepted SHA-256 working-file identities:
  `children.rs` 0BF8EF6E5C48488D720DE43D4D879079E2216EC38A84CBE1A6997FFC9AD104F2;
  `ticket.rs` A10EE3C370972B3282823B0621DF0C0FDDE5D588CC583984FCC6E0AB2D8759C5;
  `parent_cli.rs` A4B35E2DB764E656BD72DA15C59C2BEDC04246EFBF5E61450D70381A21D02F9F.

## Artifacts and limits

This record promotes the compact proof. Disposable helper and gate logs go to
OS trash after their results are recorded. Existing shared target output (~4.47 GB)
is retained for subsequent CLI work; its lifecycle ends when the shared build
cache is intentionally retired. No rendered dashboard acceptance is inferred
from this CLI change. Existing dashboard visual tickets remain open.
