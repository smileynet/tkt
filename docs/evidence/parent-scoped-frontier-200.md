# Parent-scoped frontier

`tkt ready --parent ID` intersects the existing global frontier with an existing
parent's direct children. Eligibility, ownership, machine capabilities, active
tag context and priority/ID ordering retain their existing behavior. Untagged
tickets retain their positive-context inclusion rule. JSON adds the stored parent
when present; the human WIP section is also scoped to that parent. Missing parent
IDs return NotFound (exit 1); existing tickets with no children produce an empty
frontier. This is native-parent-only: old epic-tag groups can use normal context.

- Pass: final `mise run check`, including 397 Rust tests, 42 Node fixtures passed,
  one existing Windows relative-link fixture skipped, format check and
  warning-denied clippy. Seven parent integration tests pass.
- Pass: the scoped fixture covers distinct parents, missing/empty targets,
  blocked/done/backlog/in-progress states, agent restriction, another assignee,
  machine requirements, priority/ID ordering and tag context with untagged work.
- Earlier assertions failed because test fixtures omitted required blocked_by
  and then assumed context hides untagged tickets. Fixtures and expectations were
  corrected to the observed existing contract; product filtering was not weakened.
- Independent critic accepted the final source and documentation:
  ready.rs F75D563A9542E4411A13DE7B34CC164E02BA0A856883C1451AA67FAF2FC8A3A1;
  cli.rs 4DF045E39328E50C78679BADCA4A4C1040103436CA9C9CC46340B00CB7643C6B;
  parent_cli.rs 9106D8B34CEC68E409071D4EA47CD37AA502FF34A0D4F0467E35BE8681987FC5.
- Pass: deployment precheck and scoped diff/whitespace checks. All relevant
  guidance surfaces and the driving per-map session use case are documented.

This record promotes the proof. Disposable helper and test/gate logs go to OS
trash. Shared target output remains for ongoing CLI/deployment work. This change
does not establish rendered dashboard acceptance or user outcomes.
