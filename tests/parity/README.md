# Historical fixture corpus

The unsupported Rust-versus-Python comparison runner was retired under ticket 186.
There is no supported Python reference implementation in this repository, and no
parity comparison is part of the current verification gate.

The three small fixture files remain as inputs to the maintained Rust integration
test `test_historical_fixture_contract`. It checks completed-prerequisite readiness,
full JSON fields and quote escaping against the current product contract. It does
not compare implementation output to Python or preserve obsolete output schemas.

Run the maintained gate with `mise run check`. For this fixture regression alone,
run `cargo test --test integration test_historical_fixture_contract`.
