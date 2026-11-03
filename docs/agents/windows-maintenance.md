# Windows maintenance

Read before building, formatting, installing or composing Git commits on Windows,
and when changing atomic replacement/filesystem behavior or repairing toolchain
proxies. Environment changes requiring elevation remain explicit operations.

Prepared under ticket 245. Original text is duplicated during the bounded
transition; ticket 246 removes the extracted source from AGENTS.md.

<a id="rule-32"></a>

- Windows: `std::fs::rename` replaces an existing file on supported filesystems (verified on this Windows/MSVC environment; Rust uses `MoveFileExW` with a `SetFileInformationByHandle` fallback). Do not delete a destination before an atomic replacement: that would reintroduce a missing-file window. The source and destination must remain on the same filesystem.


<a id="rule-37"></a>

- Windows (this machine): `CARGO_HOME=D:\dev-tools\cargo`, `RUSTUP_HOME=D:\dev-tools\rustup` (relocated off C:). Bare `cargo` resolves to the mise shim first, then to the rustup proxies in `D:\dev-tools\cargo\bin\` (Windows symlinks → `rustup.exe`). Prefer `mise run check` for the gate — it is PATH-order-independent. **Failure mode (seen 2026-08-31):** if `cargo fmt` dies with `os error 193` ("not a valid Win32 application"), the `cargo-fmt.exe`/`rustfmt.exe` proxy symlinks in `D:\dev-tools\cargo\bin\` have been corrupted into 0-byte regular files (cross-drive relocation / AV). `cargo build/test/clippy` still work (clippy's proxy stayed intact, and build bypasses proxies), which masks the break. Repair: delete the 0-byte files and recreate them as symlinks to `rustup.exe` (Developer Mode is on, so no elevation needed), or run `rustup-init --default-toolchain=none -y --no-modify-path`. If corruption recurs, add `D:\dev-tools` to Defender exclusions (needs an elevated shell). Verified working 2026-08-31 after symlink repair.


<a id="rule-39"></a>

- Windows: use `git commit -F <file>` for multi-line or bracket/quote-containing commit messages — inline `-m` gets mangled by PowerShell quoting
