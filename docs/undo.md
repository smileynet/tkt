# Revert a recognized ticket mutation

```sh
tkt --dry-run undo
tkt undo
```

Undo adds a Git revert commit for an eligible mutation at HEAD. Original commits
remain in history. The first implementation recognizes only `new`, `claim` and
`close` commits generated with a versioned receipt by this build. It restores the
exact committed ticket state and any recorded plan status cells, including prior
acceptance/resolution text. It does not infer ownership from the author or trust
a subject that merely resembles a tkt commit.

The index and tracked worktree must be clean, with no existing merge, rebase,
cherry-pick, revert, sequencer or index-lock state. Unrelated untracked files are
preserved. Root commits, merges, repeated undo, unreceipted legacy commits,
rebased/mismatched receipts, extra paths and unsupported file modes are refused.
Only regular non-executable `.tickets/*.md` files and an optional existing
`docs/plan.md` status change qualify. A newly tracked plan has no safe recorded
prior contents and is not recognized for undo. Unsupported commits still work as
ordinary mutations; they simply do not acquire an undo receipt.

This is a repository operation, not a claim or agent-eligibility override. It can
restore prior assignee/status fields through the recorded inverse. It does not
undo local configuration, tag context, init files, hook installation, telemetry,
advisory caches or other effects outside those committed paths. Batch/edit/
cancel/reopen/renumber and other mutation families are not recognized in this
first version. Use ordinary reviewed Git recovery for unsupported history.

## Preview, publication and failures

Preview verifies the candidate receipt and committed transition without fetching,
creating a temporary index, applying a revert, committing or pushing. Successful
preview does not prove that later filesystem operations cannot fail. Normal undo
rechecks HEAD/clean state after resolving the push-gated mutation context.

Publication honors `push.enabled` and global `--no-push`. With publication enabled,
the exact inverse is checked before and after commit and pushed once. Unlike normal
mutation retry, a rejected undo push does not automatically rebase the inverse onto
new upstream work. The local revert commit remains durable and the remote state
is preserved; reconcile explicitly before publishing.

If Git cannot apply the revert, any conflict/sequencer state is preserved. Inspect
`git status` and resolve or abort that Git operation explicitly. If the commit
fails, the staged inverse remains available for inspection. An opted-in commit
hook that changes the prepared tree leaves a local commit with an error and no
push. A publication error after a valid local commit keeps that commit. The
command never performs reset-based recovery or automatic destructive cleanup.

`-o json` reports the reverted commit, subject and preview flag; `-q` remains
quiet while `-q -o json` retains structured output.

## Receipt boundary

The canonical `Tkt-Undo` trailer binds version, subject, parent, complete before/
after trees and each supported path's blob/mode. Recognition verifies those
against actual Git objects and checks the operation's ticket/body/plan transition.
Message-only, altered, malformed, duplicated and unknown-version records fail.

Receipt creation uses a private temporary index so unrelated staged work remains
outside the mutation commit. Hooks/filters or later history rewriting can make a
receipt ineligible; the command refuses it rather than guessing. This is structural
provenance, not cryptographic author authentication. A repository writer can forge
history, so a receipt is not an authorization credential. Do not manufacture one
to bypass a refusal.
