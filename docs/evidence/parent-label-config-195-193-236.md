# Parent integrity and configuration boundaries

Separate implementers delivered #195 and #193; the coordinator wired config
errors through callers and repaired #236. Independent critic accepted the final
code, fixtures and documented syntax with no blocking findings.

Final `mise run check`: **pass**, 332 Rust tests passed, no failures/ignored;
42 Node fixtures passed with one existing Windows relative-link skip. Formatting
clean, Clippy zero warnings. Read full output; Git fixture line-ending notices
were informational. `git diff --check` passed.

- #195: five public CLI tests were red for missing checks and green after repair.
  Dangling parent, self-parent and parent-chain cycles are findings. Parent and
  blocked-by graphs are independent, including mixed-edge non-cycles and actual
  dependency cycles. Checks are wired into shared findings used by health scans.
- #193: four public fixtures cover typed namespace settings, legacy configs,
  malformed bool/list/unknown/nested fields, doctor findings and rejected mutation
  preserving ticket bytes and Git HEAD/index. Loader consumers now propagate a
  structured Validation error before mutation; health scanning reports label-config.
  Existing narrow syntax is explicit: single-line double-quoted JSON-style array,
  no trailing comma. Unsupported single-quoted/trailing-comma forms fail clearly.
  No full TOML parser is claimed; empty/duplicate value lists are representable.
  Namespace tag enforcement remains the separate newly unblocked #198.
- #236: config output/help use the loader's same path helper. Public fixture was
  red for the hard-coded path, then passed with isolated XDG_CONFIG_HOME. Native
  macOS resolution is source-inferred from that shared helper, not executed here.

No new dependency, flag, hook or version. Durable results are promoted here;
fixture directories self-clean. Retain the pre-existing shared target cache for
the ongoing queue; disposable gate logs go to OS trash after promotion. Actual
dashboard rendering remains unavailable; no owner-review request was added.
