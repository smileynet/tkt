# Approved guidance coverage map

Frozen source: AGENTS.md at 9e944bda0dbc79361976907730564b969aa033d0. Every source line is assigned exactly once
to a retained or referenced unit in [the machine inventory](guidance-coverage.json).
Original source paragraphs and bullets are preserved verbatim at their assigned
home, except relocated Markdown links are rebased and the Commands example's bare
formatter becomes a format check under
the already-existing verification rule. This map covers the migration baseline;
it does not certify future edits without review. User-level standing rules remain
untouched and continue to override local guidance.

Migration status: ticket 245 prepared homes without extraction; ticket 246 now
applies conditional pointers and ends the temporary duplication. The full local
gate, current hosted-run authorization and separate publication boundary remain
unchanged. Maintainer-tooling deliberately groups cargo-dist and generation traps
with other tool rules and links release-staging for the authoritative workflow
contract; this refines the proposal home without changing its policy. Current CLI usage is reached through commands.md/capabilities; the deep
contract keeps exact compatibility details previously held in AGENTS.md.

| Unit | Source lines | Rule or section | Final home | Read trigger |
|---|---|---|---|---|
| rule-01 | [1–2](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L1-L2) | AGENTS.md | [AGENTS.md](../../AGENTS.md) | Always loaded |
| rule-02 | [3–8](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L3-L8) | Project | [AGENTS.md](../../AGENTS.md) | Always loaded |
| rule-03 | [9–23](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L9-L23) | Workspace Layout | [AGENTS.md](../../AGENTS.md) | Always loaded |
| rule-04 | [24–35](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L24-L35) | Commands | [AGENTS.md](../../AGENTS.md) | Always loaded |
| rule-05 | [36–56](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L36-L56) | Verification gate (run before every commit) | [AGENTS.md](../../AGENTS.md) | Always loaded |
| rule-06 | [57–72](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L57-L72) | Session workflow (commit + push + deploy cadence) | [AGENTS.md](../../AGENTS.md) | Always loaded |
| rule-07 | [73–95](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L73-L95) | Deploy (run after pulling or making changes) | [docs/agents/plugin-maintenance.md](plugin-maintenance.md#rule-07) | CLI/pack installation, plugin changes or deployment diagnosis |
| rule-08 | [96–103](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L96-L103) | Releases | [AGENTS.md](../../AGENTS.md) | Always loaded |
| rule-09 | [104–145](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L104-L145) | Claude Code plugin (keep it current) | [docs/agents/plugin-maintenance.md](plugin-maintenance.md#rule-09) | CLI/pack installation, plugin changes or deployment diagnosis |
| rule-10 | [146–184](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L146-L184) | tkt CLI (the product) | [docs/agents/tkt-contract.md](tkt-contract.md#rule-10) | CLI/storage/lifecycle/configuration/plan compatibility changes |
| rule-11 | [185–194](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L185-L194) | Global flags | [docs/agents/tkt-contract.md](tkt-contract.md#rule-11) | CLI/storage/lifecycle/configuration/plan compatibility changes |
| rule-12 | [195–198](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L195-L198) | Priority levels | [docs/agents/tkt-contract.md](tkt-contract.md#rule-12) | CLI/storage/lifecycle/configuration/plan compatibility changes |
| rule-13 | [199–211](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L199-L211) | Status values | [docs/agents/tkt-contract.md](tkt-contract.md#rule-13) | CLI/storage/lifecycle/configuration/plan compatibility changes |
| rule-14 | [212–221](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L212-L221) | Query output | [docs/agents/tkt-contract.md](tkt-contract.md#rule-14) | CLI/storage/lifecycle/configuration/plan compatibility changes |
| rule-15 | [222–232](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L222-L232) | Agent eligibility | [docs/agents/tkt-contract.md](tkt-contract.md#rule-15) | CLI/storage/lifecycle/configuration/plan compatibility changes |
| rule-16 | [233–261](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L233-L261) | Configuration | [docs/agents/tkt-contract.md](tkt-contract.md#rule-16) | CLI/storage/lifecycle/configuration/plan compatibility changes |
| rule-17 | [262–275](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L262-L275) | Environment variables | [docs/agents/tkt-contract.md](tkt-contract.md#rule-17) | CLI/storage/lifecycle/configuration/plan compatibility changes |
| rule-18 | [276–287](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L276-L287) | Architecture Decisions | [docs/agents/architecture-reference.md](architecture-reference.md#rule-18) | Architecture/module/parser/Git/telemetry/error/output changes |
| rule-19 | [288–315](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L288-L315) | Contract | [docs/agents/tkt-contract.md](tkt-contract.md#rule-19) | CLI/storage/lifecycle/configuration/plan compatibility changes |
| rule-20 | [316–317](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L316-L317) | Constraints | [AGENTS.md](../../AGENTS.md) | Always loaded |
| rule-21 | [318–318](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L318-L318) | Do NOT change the frontmatter contract without updating the tkt-work skill | [AGENTS.md](../../AGENTS.md) | Always loaded |
| rule-22 | [319–319](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L319-L319) | Distributed skills (`skills/<name>/`) MUST be self-sufficient for a tkt-only adopter (someone w | [AGENTS.md](../../AGENTS.md) | Always loaded |
| rule-23 | [320–320](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L320-L320) | Do NOT change CLI flags or agent-facing behavior without updating all guidance surfaces (see `d | [AGENTS.md](../../AGENTS.md) | Always loaded |
| rule-24 | [321–321](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L321-L321) | Do NOT implement computed sort overrides for the frontier — priority+ID ordering is intentional | [AGENTS.md](../../AGENTS.md) | Always loaded |
| rule-25 | [322–322](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L322-L322) | Do NOT add dependencies beyond what's in Cargo.toml without justification | [AGENTS.md](../../AGENTS.md) | Always loaded |
| rule-26 | [323–323](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L323-L323) | Do NOT use libgit2/gix for v1 — shell out to git binary | [AGENTS.md](../../AGENTS.md) | Always loaded |
| rule-27 | [324–324](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L324-L324) | Do NOT suggest releasing with open tickets — all tickets ship first, or the user explicitly def | [AGENTS.md](../../AGENTS.md) | Always loaded |
| rule-28 | [325–325](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L325-L325) | Maintain CLI compatibility (same commands, flags, output) | [AGENTS.md](../../AGENTS.md) | Always loaded |
| rule-29 | [326–326](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L326-L326) | `cargo clippy` must produce 0 warnings; `cargo fmt --check` must produce no diff | [AGENTS.md](../../AGENTS.md) | Always loaded |
| rule-30 | [327–327](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L327-L327) | Integration tests MUST set `DO_NOT_TRACK=1` on child processes (prevents ambient env pollution) | [AGENTS.md](../../AGENTS.md) | Always loaded |
| rule-31 | [328–328](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L328-L328) | Unit tests must NOT assert specific consent state (result depends on ambient env vars) | [AGENTS.md](../../AGENTS.md) | Always loaded |
| rule-32 | [329–329](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L329-L329) | Windows: `std::fs::rename` replaces an existing file on supported filesystems (verified on this | [docs/agents/windows-maintenance.md](windows-maintenance.md#rule-32) | Windows build/format/install/commit or atomic replacement changes |
| rule-33 | [330–330](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L330-L330) | Ticket writes go through `core::atomic_write` (temp-file + rename) so a failed/interrupted writ | [AGENTS.md](../../AGENTS.md) | Always loaded |
| rule-34 | [331–331](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L331-L331) | Codex review dispatch: `codex exec --dangerously-bypass-approvals-and-sandbox` (bwrap namespace | [docs/agents/session-operations.md](session-operations.md#rule-34) | Codex review dispatch |
| rule-35 | [332–332](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L332-L332) | cargo-dist binary name is `dist` (not `cargo dist`) — `cargo dist --version` will fail; use `di | [docs/agents/maintainer-tooling.md](maintainer-tooling.md#rule-35) | Release tooling/generation or crate-name lookup |
| rule-36 | [333–333](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L333-L333) | New mutation commands MUST route push through a push-gated path (GitTransaction respects `push. | [AGENTS.md](../../AGENTS.md) | Always loaded |
| rule-37 | [334–334](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L334-L334) | Windows (this machine): `CARGO_HOME=D:\dev-tools\cargo`, `RUSTUP_HOME=D:\dev-tools\rustup` (rel | [docs/agents/windows-maintenance.md](windows-maintenance.md#rule-37) | Windows build/format/install/commit or atomic replacement changes |
| rule-38 | [335–335](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L335-L335) | Validate product changes with `cargo test` integration tests, which rebuild and exercise the cu | [AGENTS.md](../../AGENTS.md) | Always loaded |
| rule-39 | [336–336](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L336-L336) | Windows: use `git commit -F <file>` for multi-line or bracket/quote-containing commit messages  | [docs/agents/windows-maintenance.md](windows-maintenance.md#rule-39) | Windows build/format/install/commit or atomic replacement changes |
| rule-40 | [337–337](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L337-L337) | crates.io name availability: use `cargo search <name>`, NOT raw `curl` to the API (returns 403  | [docs/agents/maintainer-tooling.md](maintainer-tooling.md#rule-40) | Release tooling/generation or crate-name lookup |
| rule-41 | [338–339](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L338-L339) | Release CI is generated through `node tools/generate-release-workflow.mjs --write` using cargo- | [docs/agents/maintainer-tooling.md](maintainer-tooling.md#rule-41) | Release tooling/generation or crate-name lookup |
| rule-42 | [340–343](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L340-L343) | Parent progress | [docs/agents/tkt-contract.md](tkt-contract.md#rule-42) | CLI/storage/lifecycle/configuration/plan compatibility changes |
| rule-43 | [344–347](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L344-L347) | Transferring acceptance criteria | [docs/agents/tkt-contract.md](tkt-contract.md#rule-43) | CLI/storage/lifecycle/configuration/plan compatibility changes |
| rule-44 | [348–351](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L348-L351) | Parent-scoped frontier | [docs/agents/tkt-contract.md](tkt-contract.md#rule-44) | CLI/storage/lifecycle/configuration/plan compatibility changes |
| rule-45 | [352–355](https://github.com/smileynet/tkt/blob/9e944bda0dbc79361976907730564b969aa033d0/AGENTS.md#L352-L355) | Plan lifecycle comparison | [docs/agents/tkt-contract.md](tkt-contract.md#rule-45) | CLI/storage/lifecycle/configuration/plan compatibility changes |

Run `node tools/check-guidance-coverage.mjs` during transition and append
`--final` after extraction. The checker binds the frozen Git source, per-unit
hashes, exhaustive line partition, reachable target text and final de-duplication.
