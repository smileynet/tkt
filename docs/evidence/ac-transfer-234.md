# Explicit acceptance-criterion transfers

`tkt move-ac SOURCE --ac N --to DEST` transfers one unchecked, nonempty,
top-level single-line body criterion. N counts real checked, unchecked and moved
criteria, excluding fenced examples. The source receives a distinct `[>]` moved
marker and destination ID; the receiver receives an unchecked criterion with the
source ID and ordinal. This initial contract supports one-hop transfers; checked,
already-received and multiline criteria are rejected. Normal push/hook policy and
dry-run apply. Frontmatter validation criteria and evidence remain unchanged;
their close policy and truthful evidence obligations are not waived.

The destination is atomically replaced before the source. An interruption can
leave an unchecked source plus an orphan receiving copy; validation reports that
state. Repeating the same command recognizes the exact source back-reference and
finishes without duplication. If both writes landed before the commit, retry
commits the pair. This is recoverable multi-file publication, not a simultaneous
filesystem transaction. Conflicting edits require inspection; retry does not
overwrite mismatched receiving text. A failed push retains the local commit.

## Verification

- Pass: final `mise run check`, after the final empty-input boundary fix:
  395 Rust tests across 33 suites, 42 Node fixtures passed and one existing Windows
  relative-link fixture skipped. Formatting and clippy with warnings denied pass.
- Pass: eight `tests/ac_transfer.rs` integration tests exercise valid links,
  unchanged VC/custom/body fields, preserved unrelated index entries, repeated
  no-op calls, destination-only and both-writes-before-commit recovery, dry-run,
  invalid/empty/checked inputs, broken/duplicate/missing links, force-close
  rejection, CRLF, multiline rejection, unchanged remote publication and
  invocation-local no-push, and fenced examples with misleading headings.
- Pass: source moved markers relabelled `[x]` or `[ ]` fail validation, audit and
  close even if the receiver is deleted. Moved provenance cannot count as checked
  implementation. A receiving criterion remains unchecked until separately done.
- Independent critic accepted after fixing unchanged-upstream rejection, CRLF,
  multiline splitting, fenced examples and moved-state validation loopholes.
  Final reviewed file identities are recorded below.
  `ac_transfer.rs`: 825AC9A3D127E1494D5A342F8A8564C123FAF380BCB57EF3445B48893872C281;
  `move_ac.rs`: 6CA5007393C6ED8CCB1D68CEC0CE068D7CA42F9C9065E7C359267490BB87B40E;
  `tests/ac_transfer.rs`: 815EF7056417295B862A22643ABC32A36B1555ABEEFA99EAA290F489CC8F83F4.
- Pass: `node tools/deploy.mjs check` and `git diff --check`. Guidance updated in
  CLI snippets, core/work skills, command reference, contributor guide and README.
- Earlier full gates covered intermediate sources; they are superseded by the
  final run. A narrow test emitted an unused-import warning after the fence fix;
  it was removed before the final warning-denied gate.

## Artifact disposition and limits

This compact record is promoted; disposable helpers and superseded logs are sent
to OS trash. Shared target output is retained for ongoing CLI tickets. Repository
scratch of unknown ownership and the active LAN server state remain untouched.
No rendered dashboard acceptance or user outcome is inferred from CLI tests.
