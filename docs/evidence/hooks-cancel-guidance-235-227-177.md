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

Durable outputs are tracked source/tests/guidance and this report. Fixtures clean
their own temporary roots; promoted logs go to OS trash after use. The existing
shared target cache and pinned dependencies remain scoped to the remaining queue;
the small goal helpers expire when that queue ends.
