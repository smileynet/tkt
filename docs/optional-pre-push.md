# Optional full pre-push verification

This contributor-only convenience runs `mise run check` before publishing the
current checkout. Installation is explicit; ordinary setup and deployment do not
enable it. It requires Git, Node 22+, Mise and the project's pinned toolchains.

```sh
node tools/pre-push-check.mjs install --full-check
node tools/pre-push-check.mjs check
node tools/pre-push-check.mjs remove
```

The acknowledgement selects the full gate, which can take minutes. It does not
replace the standing requirement to inspect check output before committing.
Installation creates only an owned executable `.git/hooks/pre-push` shim (in the
common Git directory for linked worktrees). Each checkout must contain this
checker. Existing `core.hooksPath`, foreign or modified hooks and linked hook
paths are refused and preserved; integrate those systems manually. Coordinate
installation/removal with other hook writers; this is not an adversarial lock.

Git supplies the pushed object IDs to the hook. Every non-deletion ref must
resolve to the current HEAD commit; annotated tags pointing there are accepted.
Other tips and non-commit objects are refused: switch to the intended tip or
split the push. Multiple refs to HEAD run the gate once. Empty or deletion-only
input is an explicit skip. The checkout and index must be clean, including
non-ignored untracked files; skip-worktree and assume-unchanged flags are refused.
No stash, reset or checkout is performed. HEAD and cleanliness are checked again
after the gate. Repository-local Git environment variables are cleared for Mise
so the gate's disposable repositories do not inherit the pushing repository.

A failed command, missing runtime, changed HEAD/checkout or timeout aborts the
push. The five-minute deadline covers the direct Mise process on a best-effort
basis; descendants may continue. Inspect and stop them before retrying. Ignored
dependencies, the machine environment and concurrent changes between observations
are not made hermetic by these checks. Read output for assertion failures, skips
and infrastructure errors; successful execution alone does not prove correctness.

This is local convenience, not hosted CI or a security boundary. `git push
--no-verify`, removing the hook, changing its code or using another machine can
bypass it. No hook is installed in this repository as part of this implementation,
and no hosted workflow or publication is authorized by installation.

Git's [pre-push contract](https://git-scm.com/docs/githooks#_pre_push) documents
the ref input and nonzero-exit rejection behavior.

Maintainers verify this optional tool separately with
`node --test tools/pre-push-check.test.mjs`. The fixtures compile a small Rust
gate probe and exercise actual pushes to private local bare repositories; they
do not install a hook here or run hosted workflows.
