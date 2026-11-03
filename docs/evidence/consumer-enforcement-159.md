# Optional consumer enforcement guidance

The authoritative note is `docs/agents/enforcement-model.md`, reached from README
and contributor instructions. The standalone skill and command reference carry
the tkt-only recipe and limits. No workflow, hook, Rust code or release policy is
changed by this ticket.

- Pass: isolated current-target-binary probe, Node 22, fresh temporary Git
  repository. `validate --strict --brief` reports zero findings; `lint --check`
  reports all files canonical; `sync-plan --check --strict --brief` reports zero
  findings. HEAD and worktree remain unchanged after these positive checks.
- Pass: negative controls return exit 1 for a placeholder, noncanonical ID and
  canonical completed plan row disagreeing with an open ticket. Outputs name
  stub-body, a file needing lint, and plan-status-drift respectively.
- Assertion failure: the first probe expected plain `done` to trigger plan drift
  but observed pass. This was not treated as a successful gate; new ticket 248
  captures the independently reproduced defect, separate from deferred narrative
  parsing. The note explicitly records the remaining checker limit.
- Pass: both linked official GitHub pages returned HTTP 200 on 2026-11-03 and
  supported the paraphrased permission/bypass and skipped-check pending-state
  limits. The web-search service returned infrastructure errors (503); direct
  primary-page retrieval supplied the source observations instead.
- Pass: repository `validate --strict --brief`, `sync-plan --check --strict
  --brief`, deployment precheck and `git diff --check`. No Rust rebuild needed
  for this documentation-only unit; its command probe used the current target CLI.
- Independent critic accepted final enforcement note SHA-256
  4BE7F8A0A68E338490D04B1A8B50209382104BFA65027294BB9C8DBE1D165BAE.

No hosted job was dispatched. The observed legacy workflow's automatic triggers
are disclosed, not relabelled or changed. This does not grant agent authorization
to initiate hosted runs or publish a release. This proof is promoted; temporary
probe repositories, helper and logs go to OS trash. Unknown-owned scratch and the
active LAN service state remain untouched.
