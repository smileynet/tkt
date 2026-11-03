# Completed template audit evidence (#242)

## Reproduction

The public CLI regression in `tests/audit_template_content.rs` was first run
against the pre-fix audit implementation with:

```text
cargo test --test audit_template_content -- --nocapture
```

It failed as expected. The old whole-body matcher missed the current default
prompt at fixture `03` and incorrectly flagged the quoted #176 example, fenced
template `08`, and nonmatching heading `09`. This is the red reproduction of
both the missed detection and false-positive behavior.

## Fix and verification

The audit and open-ticket validation now share a Markdown-aware section reader
in `src/findings.rs`. It recognizes exact `## What to build` and
`## Acceptance criteria` headings, ends sections at the next actual level-two
heading, and skips fenced code blocks. Completed-ticket policy flags an empty
template section or an exact old/current placeholder AC line, including a
checked line or a placeholder mixed with substantive criteria. Open/in-progress
validation retains its prior policy: empty/placeholder intent or a sole
unchecked placeholder AC. Resolution migration/supersession exemptions and
status filtering remain in `src/audit.rs` unchanged.

The public regression covers old and current prompts, empty sections, checked
and mixed placeholder ACs, a quoted #176-style example, fenced examples inside
and outside sections, similar-but-not-exact headings, all three migration
exemptions, status scope, and the separate low-evidence warning. It also runs
`validate` to confirm the open/in-progress semantics with an actual prompt,
inline quoted prose, and a fenced placeholder plus nested heading. Fenced-only
intent and AC sections are considered substantive examples rather than empty;
an actual `TBD` intent remains detectable when followed by a fenced example.
Additional fixtures cover empty target headings at EOF and immediately before
another heading, repeated target headings with a later placeholder, and invalid
fence closers with trailing text or a mismatched marker.

Observed after the fix:

- `mise exec -- rustfmt --edition 2021 src/findings.rs src/audit.rs tests/audit_template_content.rs` — completed successfully.
- `cargo test --test audit_template_content -- --nocapture` — pass, 1 test, 0 failures.
- `cargo test --bin tkt findings::tests::stub_body -- --nocapture` — pass, 4 tests, 0 failures.
- `cargo fmt --check` — pass, no diff.

The #176-style fixture template false positive is removed while its intentionally
missing-evidence `low-evidence-closure` warning remains. This describes only the
fixture; it makes no claim about the historical evidence on production ticket
#176. The test demonstrates classifier behavior only and does not establish
historical verification evidence for any closed ticket. The full local gate and
independent critic review were pending at that focused-check checkpoint.

## Final combined verification

`mise run check` passed 385 Rust tests and 42 Node tests, with zero failures.
One existing Windows relative-link Node test was skipped. Formatting passed;
Clippy reported zero warnings. All four reviewed source/test identities remained
unchanged through the run. Independent critic accepted the final parser,
lifecycle policies, public regression and guidance. Native manifest validation,
four-skill inventory and deployment manifest checks passed; these checks do not
perform the directory safety/policy review.

## Production corpus classification

The explicitly addressed Cargo-built candidate ran `audit --deep --brief` against
the real board after the focused tests. It reported **11 findings: 10 warnings
and one informational notice**, down from 15 (14 warnings and one notice).
This is a successful advisory scan, not a clean audit or proof of historical work.
Strict ticket validation separately passed with zero findings.

| Finding group | Tickets | Disposition |
|---|---|---|
| Quoted-example false positives removed | 149, 152, 174, 176 | Substantive specifications quote placeholders; no historical prose or evidence was edited |
| Missing historical evidence retained | 154, 155, 156, 174, 206 | Original proof remains unavailable; no waiver or fabricated checks |
| Actual template sections retained | 48, 82, 95, 102, 129 | Existing historical dispositions remain; genuine unfilled text is still visible |
| High-priority open notice retained | 197 | Recorded parent-rollup decision gate remains unresolved |

The five template records are contextual historical debt already dispositioned in
[the closure review](historical-closure-239.md), not newly claimed product fixes.
The completed-template rule remains done-only and does not mark any work complete.
