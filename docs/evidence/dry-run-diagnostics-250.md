# Dry-run diagnostic effect repair

An independent reproduction on installed commit d197af2 used private XDG state/config roots, DO_NOT_TRACK=1 and TKT_DEBUG=log. The command tkt --dry-run config --set debug=true exited 0 and created debug.log while leaving the requested config absent. The new public regression failed on that exact effect; its ordinary-logging control passed.

The fix initializes global and local preview modes before diagnostics. The persistent debug sink checks DRY_RUN before creating directories or opening/appending a log. Explicit human/JSON stderr debugging remains available. Telemetry consent and real user configuration are unchanged.

PASS: final mise run check ran 421 Rust tests and 42 Node fixtures, with clean formatting and zero clippy warnings. SKIP: one existing Windows relative-link fixture. Two public tests cover six preview commands with absent and preexisting logs, unchanged private configuration, ordinary file logging and stderr-format controls. Every child uses DO_NOT_TRACK=1 and private roots. Independent critic review accepted the final source; diff and scope checks passed.

Parent ticket 191 remains in progress: this repair does not complete its all-mutator effect plan. The preceding completed batch at d197af2 was installed and its workflows refreshed. Claude reads the checkout directly; Codex, Kiro, OpenCode and Antigravity were refreshed. Kiro Crew was built only because its CLI is absent. This repair was subsequently installed with the final migration batch; see parked-execution-2026-11-03.md for current deployment evidence.
