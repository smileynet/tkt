# Parent controls, label enforcement and debug defaults

Separate implementers delivered #196/#198; the coordinator delivered #237 and
closure guidance #229/#230. An independent critic reviewed all final code and
guidance before acceptance.

## Checks and review

Final `mise run check`: **pass**, 340 Rust tests passed, none failed/ignored;
42 Node fixtures passed, one existing Windows relative-link skip. Formatting
clean and Clippy zero warnings. `git diff --check` passed. An earlier full run
had one **assertion failure**: an old integration test expected debug=true to
default to stderr. It now explicitly selects human format; the new config-only
test separately verifies the intended log default. The final run is after that
correction and the parent-target fix below.

The critic caught a real parent-validation gap: new/batch initially trusted
filename IDs even when the target was malformed or declared a different ID.
The accepted implementation resolves parsed local ticket IDs. Regression tests
cover both bad target forms on both creation routes. A filename alone no longer
qualifies as an existing parent.

- #196: public fixtures prove new/batch/edit parent set, edit clear, query parent
  output/filter, missing/self/cycle rejection and normal/dry-run no-write failure
  paths. HEAD, index/status and edited bytes stay unchanged. Capabilities derives
  the new flags; all six generated instruction targets and guidance are updated.
- #198: four public fixtures verify closed values, single-select cardinality,
  independent axes, undeclared/free-form tags and empty closed-set behavior.
- #237: config-only debug fixture was red before the default change, then passed
  with actual log bytes and clean stderr. Explicit human/JSON overrides still
  produce their selected stderr format. Config and state are isolated per fixture.
- #229/#230: source review covers all five snippet constants (six output targets),
  work/reconcile guidance, classified command/output evidence, artifact versus
  tool/human/user-outcome distinctions, unavailable checks, scoped retention and
  removal of expired owned artifacts. No external skill dependency was added.

## Artifact disposition and limits

Durable outputs are source/tests/guidance and this evidence record. Test fixture
directories self-clean. Disposable gate logs/scripts are sent to OS trash after
promotion; the small commit-message file is retained only for this ongoing goal.
The pre-existing shared target cache is retained for subsequent ticket gates,
with cleanup due when that shared verification scope ends; it is mixed-owner and
was not copied. No historical proof or user outcome is inferred from these checks.
Local Windows execution does not verify other operating systems. Dashboard
rendering remains unavailable and creates no owner-feedback request.
