# Update-check interval verification

Implemented user-only update_check.interval with positive whole hours/days or never; default24h. TKT_UPDATE_CHECK_INTERVAL overrides config; existing privacy/quiet/dry-run/CI/TKT_UPDATE_CHECK=0 guards execute before lazy config resolution and cache access. Malformed manual/env values warn and skip; config writes reject invalid values and project scope before mutation.

- PASS: final mise run check: 401 Rust tests; formatter clean; clippy zero warnings;42 Node fixtures passed.
- SKIP: one existing Windows relative-link migration fixture.
- ASSERTION FAILURE, repaired: first gate's isolated privacy harness lacked the newly referenced config module. Updater now accepts a lazy resolver; production main supplies actual configuration. Final gate rerun passed.
- PASS: real updater module compiled in isolated fake-curl harness; DNT/CI/update-disable guards prevent resolver and cache effects; never/malformed suppress cache root; expired1h/no-cache fetch only fake curl; fresh24h/7d preserve cache. No real networking or user consent change.
- PASS: independent critic accepted final main97E99095…127FBC97, updaterCDDA2790…FFA4D06E, privacy harness46E67733…8DC56674; reviewed configuration and guidance too.
- PASS: deploy manifest check; Claude strict manifest validation found nothing to report within its stated scan limits; inline inventory lists all four skills.

Live crates.io availability and real outbound networking were intentionally not exercised; the change uses the existing fetch path. Deployment is recorded after source commit.
