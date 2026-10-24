---
id: "215"
title: "Make the declared Rust minimum pass a locked build"
status: done
blocked_by: []
priority: high
validation_criteria:
  - "A hosted locked binary check reproduces the Rust 1.80 dependency failure at the audited source SHA."
  - "A hosted locked binary check passes at the declared minimum Rust version without ignore-rust-version."
  - "The manifest and README source-build prerequisite agree with the enforced CI minimum."
tags: ["stream:tooling"]
---

# Make the declared Rust minimum pass a locked build

## Intent source

Found during grey-matter release-readiness audit 70, implementing the user's
request to address outstanding findings and stage tkt 0.5.0 without publication.

## What to build

Source users can build the locked project with its declared minimum Rust version.
CI enforces that support contract and the README states the same prerequisite.
Keep CLI behavior and binary platforms unchanged.

## Context and research

Read `Cargo.toml`, `Cargo.lock`, `.github/workflows/ci.yml`, then `README.md`.
At source `b2d2321`, package rust-version is 1.80; locked clap 4.6.4,
clap_builder 4.6.2, clap_derive 4.6.4 and clap_lex 1.1.0 require Rust 1.85
and edition 2024. Dev getrandom 0.4.3 also requires 1.85. Metadata establishes
the incompatibility; no 1.80 compiler invocation has run yet. Lockfile v4 itself
is supported by Cargo 1.80.

[Audited evidence and sources](https://github.com/smileynet/grey-matter/blob/5286232/docs/agents/research/tkt-msrv-evidence.md).
Primary references: [Cargo rust-version](https://doc.rust-lang.org/stable/cargo/reference/rust-version.html),
[Rust 2024 release](https://doc.rust-lang.org/edition-guide/rust-2024/index.html),
[Cargo changelog](https://doc.rust-lang.org/stable/cargo/CHANGELOG.html#cargo-178-2024-05-02).

First reproduce with hosted `cargo +1.80.0 check --locked --bin tkt`, retaining
exact SHA and diagnostics. Then either raise the honest minimum (1.85 is the
metadata-supported candidate) or deliberately pin compatible dependencies with
justification. Prove the selected minimum on a hosted locked build; don't use
ignore-rust-version or claim metadata inspection is compiler execution.
No toolchain install on the disk-constrained local workspace. This ticket does
not publish anything or authorize an unrelated dependency update.

## Acceptance criteria

- [x] Hosted Rust 1.80 locked binary check retains exact failing SHA and diagnostic.
- [x] Hosted locked binary check passes on the declared minimum.
- [x] Manifest, README and CI declare the same supported minimum.

## Resolution (2026-10-24)

done: declare Rust 1.85.0 and enforce a hosted locked binary check from the manifest; document the build prerequisite and upgrade path. The separately discovered cold-cache fixture failure is tracked by 216.

### Verification
1. ✓ A hosted locked binary check reproduces the Rust 1.80 dependency failure at the audited source SHA. — "Hosted run 40054220986 job 120753486806 at 3627d815130b2a43e3842dd6104d7ede5adaa275 used Cargo 1.80.1; cargo check --locked --bin tkt exited 101 on clap_lex edition2024."
2. ✓ A hosted locked binary check passes at the declared minimum Rust version without ignore-rust-version. — "Hosted run 40054566229 job 120754556480 at a1fe66a7bb7e096ddf8f98fbe4d0f6779b5a5087 succeeded with exact Rust 1.85.0 and cargo check --locked --bin tkt."
3. ✓ The manifest and README source-build prerequisite agree with the enforced CI minimum. — "Cargo.toml declares 1.85.0; README source/crates.io prerequisites agree; CI derives toolchain from that manifest. Local mise run check passed 14 Node and 296 Rust tests, fmt and clippy."
