# Update-check interval verification

Implemented user-only update_check.interval with positive whole hours/days or never; the default remains 24h. TKT_UPDATE_CHECK_INTERVAL overrides configuration. Existing privacy, quiet, dry-run, CI and TKT_UPDATE_CHECK=0 guards run before lazy configuration resolution and cache access. Malformed manual/environment values warn and skip; config writes reject invalid values and project scope before mutation.

PASS: final mise run check ran 401 Rust tests and 42 Node fixtures, with clean formatting and zero clippy warnings. SKIP: one existing Windows relative-link fixture. ASSERTION FAILURE, repaired: the isolated privacy harness could not compile after the updater gained a config dependency. The updater now receives a lazy resolver; production supplies actual configuration and the harness verifies guards before resolution.

Isolated fake-curl cases cover disabled/malformed checks, expired hourly data, fresh daily/weekly data and absent cache. No real network or consent change was used. Public config tests cover defaults, valid/invalid values, dry-run, project rejection, environment precedence and unset. Independent critic review accepted the final source. Manifest checks passed within their reported limits. Real crates.io availability was not exercised; deployment succeeded in the completed batch.
