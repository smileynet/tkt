# ADR 0003: audit-debt baseline — grandfather the pre-protocol corpus

- Status: accepted
- Date: 2026-10-19
- Source: v0.5.0 review (handoff open question A1; ticket #156 finding D1), decided by user

## Context

`tkt audit --strict` reports 30 findings on this repo's own ticket corpus
(measured 2026-10-19, HEAD on `main` at v0.4.0 post-release):

- **27 × `missing-resolution`** — done tickets with no `## Resolution` section
  (`src/audit.rs:12-28`, `check_resolution_quality`). These are tickets 02–29 and 35,
  which were closed *before* the close protocol and Resolution-section convention
  existed. Evidence for that work can no longer be reconstructed honestly.
- **3 × `low-evidence-closure`** — #154, #155, #174, closed with evidence thinner than
  the current gate would accept.

`tkt validate --strict` and `tkt sync-plan --check` both pass; this debt is isolated to
`audit`. The handoff explicitly asked that this not be treated as a silent release
blocker without a recorded policy. Ticket #156 finding D1 raised the same question and
recommended a grandfather cutoff over fabricating retrospective resolutions.

The three options considered (from #156 D1):

1. **Grandfather cutoff** — exempt done tickets below an id/date threshold. Cheapest,
   honest; the exempted tickets are genuinely pre-protocol.
2. **One-time retrospective Resolution stubs** — fabrication risk; the close gate
   explicitly forbids `validate --fix` from fabricating resolutions, and a manual sweep
   carries the same honesty hazard.
3. **Config exemption list** — per-ticket allowlist in `.tickets/config.toml`; accurate
   but verbose and manual.

## Decision

1. **`audit --strict` is NOT a v0.5 release blocker in its current state.** The 30
   findings are documented, understood debt, not a correctness regression. The release
   gate stays `mise run check` + `validate --strict` + `sync-plan --check`.

2. **Grandfather the pre-protocol corpus (option 1) as the baseline policy.** Done
   tickets that predate the close protocol are exempt from `missing-resolution` /
   `tbd-resolution`. The exemption is a **cutoff**, not a fabrication: we never invent
   evidence for work whose evidence is gone.

3. **Implementation is deferred to its own ticket**, not done in this review. The
   implementation ticket owns the mechanism choice (id threshold vs. closed-before date
   vs. config key) and must:
   - make the cutoff explicit and configurable (a `.tickets/config.toml` key under
     `[audit]`, defaulting to the known pre-protocol boundary for this repo),
   - apply only to the resolution-quality rules, never to `low-evidence-closure` or AC
     completeness,
   - have regression coverage proving a post-cutoff done ticket without a Resolution
     still fires.

4. **The 3 `low-evidence-closure` findings (#154/#155/#174) are NOT grandfathered.**
   They were closed under the current protocol; the thin evidence is a real (if minor)
   quality gap. Disposition: accept as known debt for v0.4.0 (already shipped), and do
   not ratchet `audit` severity on them until the cutoff mechanism above lands. If any
   is cheaply re-verifiable, a future close-quality pass may backfill evidence — but
   that is opportunistic, not a gate.

## Consequences

- `tkt audit --strict` continues to report 30 findings until the cutoff ticket lands;
  this is expected and recorded here, so a fresh session does not mistake it for a
  regression or silently "fix" it by hand-editing `status`/Resolution.
- Once the cutoff ships, `audit --strict` should drop to the 3 `low-evidence-closure`
  findings, making the signal usable as an advisory quality check again.
- The grandfather cutoff is a one-way honest baseline: it admits the pre-protocol
  corpus can't be reconstructed, rather than papering over it with fabricated stubs.
- #156 finding D1 is resolved by this ADR (the baseline decision it asked for);
  #156's other findings (D2–D5) concern #154's spec, which is already closed — see the
  #156 disposition.

## Alternatives rejected

- **Retrospective Resolution stubs (option 2):** rejected — fabricating evidence for
  work whose real evidence is gone is exactly what the close gate forbids; it would make
  `audit` lie rather than report honest debt.
- **Treat the 30 findings as a release blocker:** rejected — they are pre-existing,
  understood debt on already-shipped work, not a v0.5 correctness regression; blocking
  on them would stall the release for no integrity gain.
- **Ratchet `low-evidence-closure` to an error now:** rejected — it would retroactively
  fail v0.4.0 work; revisit only after the cutoff mechanism exists.
