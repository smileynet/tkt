# LAN dashboard hosting — 244

## Running instance

- Address: `http://192.168.0.187:8765/docs/status.html`
- Ethernet address/gateway observed: `192.168.0.187` / `192.168.0.1`; profile Private.
- Current Node PID and supervised session are recorded in `.scratch/dashboard-lan.json`.
- Command: `node tools/serve-dashboard.mjs --host 192.168.0.187 --port 8765`.
- Lifetime: until stopped, supervising session ends, or the machine restarts.
  No boot task or Windows service was installed.

The server exposes only the dashboard and its explicitly linked local resources,
inside the checkout. It rejects unrelated repository files, traversal paths,
Git/scratch/reference metadata and writes. Source is read per request with
`Cache-Control: no-store`. No new runtime dependency was added.

## Verification

- PASS: `node --test tools/serve-dashboard.test.mjs` — two public HTTP tests,
  zero failures. Covers redirect, matching content, linked evidence, missing
  paths, encoded traversal, method restriction, HEAD, fresh content after edits,
  and linked symlinks to Git metadata or outside the checkout. No tests were skipped.
- PASS: `node --test tools/serve-dashboard.test.mjs tools/dashboard-decisions.test.mjs`
  — nine tests, zero failures/skips on the final code, including case-insensitive
  metadata-path rejection. The supervised server was restarted with that code.
- PASS: proxy-disabled .NET HTTP request to the LAN address returned 200 and
  `text/html; charset=utf-8`; its SHA256 matched `docs/status.html`.
- PASS: live requests for `/.git/config`, `/Cargo.toml` and
  `/.tickets/config.toml` returned 404. A subsequent request also returned 200.
- PASS: existing enabled inbound firewall rules match the actual Node executable,
  permit Private networks, and include TCP with any local port. No rule was changed.
- Observation gap: Get-NetTCPConnection/netstat did not enumerate the listener,
  although direct HTTP succeeded and CIM identified the Node process/command.
  The observed HTTP response takes precedence over that listing omission.
- UNKNOWN: no second physical device was controlled to test Wi-Fi routing or
  client isolation. Browser rendering is not inferred from HTTP success; previous
  UI acceptance remains pending. Plain HTTP may use the existing clipboard fallback.

Automatic approval review rejected the initial detached Start-Process launch with
only `blocked by policy`. A supervised foreground execution was accepted and is
the running instance. No rejected background-launch route was retried.

## Artifacts

The server, public regression, hosting guide and this report are tracked outputs.
Test fixtures self-clean. `.scratch/dashboard-lan.json` is retained only while
the running instance needs its PID/session lifecycle record; remove it after the
instance stops. Temporary ticket/dashboard-update helpers are recycled after
publication. Existing unknown-owner scratch and shared caches retain the measured
scopes in the maintenance report and were not deleted.
