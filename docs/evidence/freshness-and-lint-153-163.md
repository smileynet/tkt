# Fresh validation and missing-ID repair

Tickets #153 and #163 were implemented by separate workers and accepted by an
independent critic, including a follow-up review of the final guidance, gate and
canonicality assertion. No blocking findings remained.

## Observed verification

- `mise run check`: pass on the final implementation. Node: 42 passed, no
  failures, one existing Windows relative-link fixture skipped. Rust: 315 passed,
  none failed or ignored; formatting clean, Clippy zero warnings. Git fixture
  LF/CRLF conversion notices are informational, not compiler warnings.
- Freshness public wrapper fixtures: matching hash exits 0 with JSON `fresh`,
  mismatched hash exits 1 with JSON `stale`; missing executable and non-repository
  implementation fixtures exit 2 with JSON `error`. Windows wrapper tests exercise
  paths containing spaces. Product validation uses Cargo's rebuilt integration
  binary; installed binary checks are reserved for installation/deployment.
- Missing-ID integration fixtures were red before implementation and pass after
  repair. Numeric-prefix filenames produce quoted IDs; the remaining candidate
  must parse as a valid ticket before atomic writing. Unknown fields and body are
  preserved. Repaired files pass `lint --check` and `validate`. Check/dry-run leave
  bytes unchanged; existing IDs survive; invalid filenames/statuses remain untouched.

## Scope and limits

The old ticket narrative incorrectly claimed ordinary reads already derive IDs.
Observed behavior was a missing-required-ID parse failure; only lint now repairs
this bounded case. Existing-ID files retain normal lint canonicalization behavior.
No new CLI flag, hook, status contract or version was introduced. Bash requires
Node, already a maintainer prerequisite; the installed Rust CLI remains standalone.
This verifies Windows locally, not execution on macOS/Linux or rendered dashboard
layout. Browser checks remain unavailable and do not become owner-feedback asks.

Fixture temporaries self-clean. The shared pre-existing target cache is retained
for the remaining ticket queue; no build tree was copied. The disposable gate log
is promoted into this result and may be trashed after closure.
