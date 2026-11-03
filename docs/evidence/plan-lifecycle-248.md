# Exact plan lifecycle comparison

The enforcement probe reproduced a plain `done` row passing against an open
ticket. The old checker reduced state to a done emoji or cancelled prefix.
It now parses all five lifecycle labels, accepting plain `done` and canonical
`✅ done`, comparing actual status and cancellation reasons. Unknown cells produce
`plan-status-invalid`; fix regenerates them. Narrative parsing remains deferred.

- Pass: targeted eight sync-plan integration tests, including two new table-driven
  tests covering seven mismatched/invalid cells and six matching cells. They check
  every lifecycle, cancellation detail, strict/check versus advisory exits,
  fix/dry-run, unchanged matching bytes, HEAD and surrounding prose.
- Pass: final `mise run check`: 399 Rust tests, 42 Node fixtures passed, one
  existing Windows relative-link fixture skipped, format and warning-denied clippy.
- Pass: current target CLI checks this repository's complete plan and ticket
  corpus with strict validation/check mode, zero findings; deployment precheck
  and `git diff --check` pass.
- Independent critic accepted source SHA-256
  sync_plan.rs 2CA1FEFF368DBB7F68075B8D56B198355EB096C5A2A26631FE13626F7FE7F7CD;
  integration.rs C9513ED5E19F3CA82867D958EE928A1F836734E8670A94CF626ACEBFC136C331;
  init.rs E108D412B35BA92F84EA87C2D36D32A79D7387D2E7EE36DD55FF57F843366E14.

Guidance and the prior enforcement note now describe the repaired behavior.
No CI workflow or narrative format was added. This proof is promoted; disposable
brief/helper/test/gate logs go to OS trash. Shared target output remains for
installation and subsequent maintenance; unknown-owned scratch is untouched.
