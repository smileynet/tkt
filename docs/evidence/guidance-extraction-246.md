# Conditional contributor guidance

The runbook now has 139 physical lines, retaining operations and safety rules.
The frozen inventory's 45 units cover every one of its 355 source-line positions;
reference details moved to their verified homes. Root task pointers cover every
home. Full verification, commit/fetch/push/deploy, close-only/body ownership, plan
synchronization, push-race/local-publication distinctions, skill ownership and
release authority remain inline. No CLI behavior or version changed.

- Pass: `node tools/check-guidance-coverage.mjs --final` and its default final
  mode, including source/unit identity, exhaustive coverage, exact destination
  text, local links, root reachability, de-duplication and human-map consistency.
- Pass: three controlled negative mutations each reject: removing a root
  reference pointer, restoring extracted duplicate text, and altering a human
  table trigger. Original bytes restored; final check passes again.
- Pass: Node syntax check, scoped diff/whitespace check, strict ticket validation
  and strict plan check; deployment precheck, strict native Claude manifest
  validation and native plugin inventory (four workflow skills) pass.
- Independent critic exhaustively accepted the final extraction and triggers,
  finding no weakened verification, mutation or release rules. Accepted SHA-256:
  AGENTS.md 7494EB60D4DC3C832FFC7DABC3636C93FD69DE22B90E856755DD376A8E4EF0F0;
  coverage JSON 09C72BE2F3ACC4E500C445D6C4EAB96A30E1DDDE02AEA1D832F4D2B5900EEC37;
  checker 5AFF0567C602F66AEB83D40BFED6DEB6C210C678C5DEE2A4BECBFA472AE10251;
  guidance matrix D0E8010DB6CC4EA8D29F664781F80A3DA58B37328A20F28EBCA2DAAC5D655D8A.

Only allowed semantic-neutral normalizations apply: relocated Markdown targets
are rebased, and the old bare formatter example becomes `cargo fmt --check` under
the already-existing gate rule. Maintainer-tooling's home refinement is recorded
and explicitly links authoritative release-staging guidance. The bounded duplicate
transition ended. Future edits still require review; a frozen map is not proof of
new behavior. Stage 247 audits and deploys this final structure.

The compact record is promoted; disposable application/negative-control helpers
go to OS trash. No build output was created. Shared target output remains for
the final CLI deployment, and unknown-owned artifacts remain untouched.
