# Agent discovery and setup — ticket 127

The prerequisite manifest (#138) is complete. Top-level help now points to
`tkt capabilities -o json`; normal initialization recommends project instructions,
separately installed workflow skills/plugins and structured discovery. Every
embedded target retains the pointer. CLI installation is not described as
installing plugins or skills.

## Verification

- Public `tests/onboarding.rs` began with four assertion failures, then passed
  for root help, default initialization, all six generated instruction files and
  snippet-only/quiet behavior. Temp repositories contain no copied project data.
- Final `mise run check`: **pass**. Rust: 193 unit, 4 agent eligibility,
  2 capabilities, 6 deployment, 98 general integration, 4 onboarding and
  4 plugin-manifest tests = **311 passed**, none ignored. Formatting check had
  no diff and Clippy had zero warnings. Node: 38 passed, one existing Windows
  relative-link fixture skipped. Git fixture LF/CRLF notices were informational;
  global Git settings were not changed.
- `cargo install --path . --locked`: pass. `tkt --version` reported
  `0.4.0 (30e3182)`, matching implementation HEAD when probed. Installed help
  contained `For machine-readable command schemas: tkt capabilities -o json`.
- In a new local Git fixture, installed `tkt init` created its project files and
  printed recommendations for `tkt init --write`, separate workflow plugins or
  source-checkout deployment, the explicit CLI-only installation distinction,
  structured command schemas and the project setup guide.
- `claude plugin validate .claude-plugin/plugin.json --strict`: pass, nothing
  to report. `claude --plugin-dir . plugin details tkt`: four expected skills
  and the existing Claude-only SessionStart hook. These are local native checks,
  not a catalog safety/policy review or published release certification.

## Standards review

Independent worker reviewed `b70a10b...30e3182` against AGENTS.md, the guidance
surface checklist and the app/data-modeling rubric. No actionable findings.

## Spec review

Independent worker reviewed the same immutable diff against #127 and the completed
#138 prerequisite. No actionable findings. The former vague effectiveness AC was
made observable through generated-target pointers, correct installation guidance
and preserved quiet/snippet contracts, without claiming a study of agent outcomes.

## Limits and retention

Runtime checks ran on Windows/MSVC. Other OS/harness runtime behavior and remote
release channels were not tested or published. Version fields stay unchanged;
changed embedded snippets reach this machine by local install and other users
through coordinated release delivery.

The final gate summary, native output and install observations are promoted here.
Own temporary installer fixture, editing/dashboard scripts, gate log and commit
messages are sent to OS trash. The pre-existing shared maintainer target cache
stays for continued development and follow-on #153 checks; reassess before cleanup.
