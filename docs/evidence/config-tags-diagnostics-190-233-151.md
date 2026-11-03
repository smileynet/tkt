# Typed configuration, editable tags and corpus diagnostics

Independent implementers delivered #190/#233; the coordinator delivered #151.
Independent critic accepted final code and guidance after two real config fixes:
config commands remain available to inspect/repair malformed user debug settings,
and effective environment overrides take precedence over stored debug values.

Final `mise run check`: **pass**, 350 Rust tests passed, no failures/ignored;
42 Node fixtures passed, one existing Windows relative-link skip. Formatting
clean, Clippy zero warnings; `git diff --check` clean. Focused tests began red
for the documented behaviors and passed after repairs.

- #190: seven public policy fixtures cover malformed push/evidence/priority/env/
  cutoff values, contextual key/value/source errors, project-over-user precedence,
  unknown-key warnings, config diagnostics/repair and effective debug overrides.
  Boolean aliases true/false/1/0/yes/no remain valid. Evidence is an enum; optional
  priority/env/cutoff values are typed. Consumers validate before mutation, never
  silently interpret a typo as a permissive gate. Existing audit string-parser
  tests moved to shared configuration/public boundary coverage with typed cutoffs.
  Config set/unset/get/list remain usable for repair rather than being blocked by
  global startup validation. Writer/project-scope additions remain #165.
- #233: four public edit-tag tests plus capabilities coverage prove replacement,
  clear, unique additions and final removals, comma/repeated forms, serialization,
  dry-run no writes, preserved context/eligibility/requirements and commit detail.
  All generated instruction targets and relevant guidance document the ordering.
- #151: public fixture proves empty and skipped-invalid corpuses are distinguishable
  through stderr; `-o json` emits one warning object per line, human warnings use
  ASCII-aware symbols, valid rows remain unchanged and skip/continue remains
  nonfatal. This does not pretend an exit-0 corpus with skips is healthy.

Durable results are source/tests/guidance and this report. Fixture temporaries
self-clean; disposable logs go to OS trash after promotion. Pre-existing mixed
target cache retained for the remaining goal: 4,158,815,845 logical bytes measured;
existing pinned node_modules retained: 3,934,146 bytes. Neither is an owned
disposable directory to delete blindly. The small dashboard-update and commit
message helpers remain scoped to this ongoing goal and expire when it ends.
Windows execution does not establish other-platform runtime or rendered UI.
