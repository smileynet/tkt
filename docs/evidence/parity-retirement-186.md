# Historical parity retirement

The three historical fixture files existed; the old ticket claim that they were absent was incorrect. All remain, now consumed by test_historical_fixture_contract in tests/integration.rs. The unsupported tests/parity/run.sh was sent to the Windows recycle bin; git records its deletion. README and workspace inventory no longer give Python comparison instructions or obsolete simplified-ready expectations. No supported Python implementation exists here and no historical parity result is claimed.

Maintained coverage: new regression parses query and ready JSON, verifies quote escaping, full optional fields, priority/env and completed-prerequisite readiness. Existing test_validate_finds_issues, test_validate_passes_clean_corpus, self/two-node-cycle and acyclic regressions cover validation intent from the old runner.

PASS: final mise run check: 402 Rust tests,42 Node fixtures; fmt clean, clippy zero warnings. SKIP: existing Windows relative-link fixture. PASS: git diff --check. Independent critic accepted retirement after ticket correction. No Python runner executed, no Python parity claimed.
