# Complete capabilities manifest — ticket 138

## Result

The command and argument inventory comes from Clap; domain overlays describe
supported values and mutation modes. There are 22 advertised commands, including
the built-in help dispatcher. Canonical `validation` and compatible `vc` aliases,
array/delimiter inputs, required fields, optional-value flags, conflicts, global
flags and output formats are documented in the
[manifest reference](../../skills/tkt-core/references/capabilities.md).

No runtime mutation handler or CLI flag changed. The supported schema does not
advertise generic done-status writes; a separately reproduced existing bypass
is tracked in #241, with the fixture's observed bytes recorded in that ticket.

## Verification

- Public CLI regression began red: `cargo test --test capabilities -- --nocapture`
  failed on missing output metadata. The finished tests compare the actual help
  command/flag surface and explicit domain cases against the generated manifest.
- An initial full gate failed on test-only regex compilation inside a loop. The
  regex was moved outside the loop; no lint suppression was added.
- Replacement `mise run check`: **pass**. Node fixtures: 38 passed, one existing
  Windows relative-link fixture skipped. `cargo fmt --check`: pass, no diff.
  Clippy: pass, zero warnings. Rust: 193 unit + 4 agent eligibility + 2 capabilities
  + 6 deploy + 98 integration + 4 manifest tests = **307 passed**, none ignored.
- `cargo install --path . --locked`: pass. Immediately after installation,
  `tkt --version` reported `0.4.0 (62a40a3)`, matching Git HEAD. Installed
  `tkt capabilities -o json` returned 22 commands, edit statuses
  open/in_progress/backlog and `vc.alias_of = validation`.
- Strict ticket validation and strict plan checks passed with zero findings.

These are local Windows/MSVC checks, not cross-platform or hosted-CI certification.
Later bookkeeping commits do not change the reviewed Rust source identity.

## Standards review

Independent collaboration worker reviewed immutable `c2ac597...62a40a3` against
AGENTS.md, recorded ADRs, the app smell rubric and data-modeling lens. No actionable
Standards findings; no finding tickets filed on this axis.

## Spec review

Independent collaboration worker reviewed the same immutable diff against #138
and #127's boundary. No actionable Spec findings; no finding tickets filed on this
axis. Its debug-binary hash differed, so it correctly limited its verdict to source
and test inspection. The root agent's installed-binary observation above separately
binds runtime metadata to `62a40a3`.

## Retention

The compact reproduction and command results are promoted here and in #241.
The new local-only status fixture, editing script and temporary commit message
are disposable and sent to OS trash before closure. The pre-existing, shared
maintainer Rust target cache is retained for follow-on #127/#153 verification;
reassess it after those tasks rather than deleting mixed-owner build data.
