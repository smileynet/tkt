# Scoped ticket commits, retirement and content guidance

Independent workers delivered #235/#227, reviewed by a separate critic. The
coordinator synchronized #177 and the canonical dashboard status model.

`mise run check`: **pass**, 375 Rust tests passed, no failures/ignored; 42 Node
fixtures passed with one existing Windows relative-link skip. Formatting clean
and Clippy zero warnings. The Git-backed integration group took **554.64 s** in
this run, unusually slow versus earlier runs; its assertions passed, but the
performance cause is unverified. Do not infer a performance result from exit 0.
After documentation-only reconcile/orient refinements, applicable Node fixtures
were rerun: 31 passed, one existing skip. `git diff --check` passed.

- #235: seven public fixtures cover skipped failing/sleeping commit hooks,
  commit.hooks=run opt-in, --only pathspec commits that preserve unrelated staged
  and untracked files, no-push skipping fetch/push, rejected pre-push hooks with
  local SHA/output/retry instruction, and deferred-ID collision recovery through
  rebase before later push. Commit hook bypass is invocation-local; pre-push hooks
  remain active. No global Git configuration changed.
- #227: seven public fixtures cover reason/replacement validation, cancellation
  versus done, explicit active-dependent actions/cycle refusal, reopening/history,
  validation/audit/plan/query/capabilities, and local push-disabled commits.
  Terminal dependents' obsolete edges are dropped and named automatically, while
  bodies, criteria and resolution prose stay intact. Actual board retirements
  are a separate required acceptance step, to be recorded after deployment.
- #177: all six generated instruction targets and core/work/orient/reconcile,
  command/format/standards references, README and AGENTS distinguish body AC
  checkboxes from indexed frontmatter evidence with independent counts. Creation
  body/source citation and tag replace/clear/add/remove guidance matches runtime.
  No version bump was performed. Independent critic corrected stale reconcile
  instructions that had treated retirement as completion.

Canonical dashboard integration is commit `cec1682` in grey-matter. Model and
review-receipt fixtures: **53 passed**. Committed pre-merge: dashboard-contract
and review-receipt both passed. Cancelled stays distinct from done, outside
remaining work and unsatisfying as a prerequisite. Receipt status normalization
recognizes cancelled, but cancellation prose remains reviewed source. Both local
dashboards adopted the canonical renderer preserving all three JSON blocks and
their checkers passed with zero warnings. Actual rendering remains unavailable,
so UI tickets #228/#240 stay pending and no feedback request is added.

The initial grey pre-merge attempt selected zero uncommitted changes (**skip**,
not a pass). Grey pre-push later failed doc-links on three unrelated research
documents' scratch citations; that shared branch also contains other unpublished
work. Its push/deployment is held without modifying or publishing those paths.
No waiver or fabricated pass is recorded.

That publication blocker was then resolved by an isolated worktree based on
origin/main: only the nine reviewed dashboard paths were committed as `0604d8d`.
Its pre-merge checks passed (2/2) and pre-push checks passed (6/6 selected),
including doc-links. The isolated commit was pushed to grey-matter main without
the other workstream's commits. It was merged back into the shared checkout;
the sole content difference was generated snapshot time, verified equivalent
after removing that metadata field. Skill source/manifests match published main,
and local deployment check/refresh passed. Original unrelated research history
was neither rewritten nor published by this task.

## Installed and live-board acceptance

`cargo install --path . --locked` passed; the freshness wrapper reported JSON
fresh with installed/head `01ec62a` before board actions. Native Claude strict
manifest validation passed and details listed four skills plus SessionStart;
this does not establish the separate catalog safety/policy review.

Each approved retirement was previewed, then applied with `tkt cancel`: #208 as
duplicate of #138, #87 as superseded by #176, #115/#161 obsolete and #94 wontfix.
All had zero dependents. Source control comparison verified each original body's
content is preserved before Cancellation (the append normalizes trailing whitespace); each commit
contains only its ticket and existing plan. No AC was checked or Resolution
invented. #142 was also retired as wontfix after a fresh bounded fixture returned
plain/either tickets 01/02 and correctly excluded GPU-required 03 under
CREW_ENV=either. This declines an unproven current defect; the historical report
and unspecified AC cases remain unverified, and it can be reopened with a concrete
counterexample. #140 is already completed with evidence. #69 stays deliberately
backlog with its explicit integration decision gate: broader research is not
claimed complete and owner input remains needed to choose that scope.

After the six retirements, strict validate and strict plan sync both passed with
zero findings. The canonical renderer migration remains source/model evidence;
the UI acceptance limitations above are unchanged.

Durable outputs are tracked source/tests/guidance and this report. Fixtures clean
their own temporary roots; promoted logs go to OS trash after use. The existing
shared target cache and pinned dependencies remain scoped to the remaining queue;
the small goal helpers expire when that queue ends.
