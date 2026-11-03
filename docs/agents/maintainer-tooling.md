# Maintainer tooling traps

Read when inspecting cargo-dist, checking crate-name availability or changing
generated release workflows. [Release guidance](../releasing.md) controls
staging/publication authorization; this reference grants none.

Prepared under ticket 245. Original text is duplicated during the bounded
transition; ticket 246 removes the extracted source from AGENTS.md.

<a id="rule-35"></a>

- cargo-dist binary name is `dist` (not `cargo dist`) — `cargo dist --version` will fail; use `dist --version`


<a id="rule-40"></a>

- crates.io name availability: use `cargo search <name>`, NOT raw `curl` to the API (returns 403 "violation of API data access policy" for automated requests); a 404 on the web page also confirms availability


<a id="rule-41"></a>

- Release CI is generated through `node tools/generate-release-workflow.mjs --write` using cargo-dist 0.32.0. Never hand-edit `.github/workflows/release.yml`; `mise run check` checks real generation. See `docs/release-staging.md`. Staging is build-only; the separate manual `publish.yml` owns reviewed publication. Final candidate/install/audit/staging gates remain open.
