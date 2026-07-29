# Parity Test Harness

Compares Rust `tkt` output against the Python `tkt` implementation for read-only commands.

## Running

```bash
# Both tools on PATH
tests/parity/run.sh

# Custom paths
TKT_RUST=./target/release/tkt TKT_PYTHON="python tools/tkt/tkt/cli.py" tests/parity/run.sh
```

## What's compared

| Command | Notes |
|---------|-------|
| `ready` | Frontier output (human format) |
| `ready --json` | JSON Lines row schema |
| `validate --brief` | Human-readable findings |
| `validate` | JSON findings |
| `query` | Full corpus JSON Lines |

## Known intentional divergences

- `ready --json` — Rust currently emits simplified rows (id, title, status). Python emits full rows with all fields. This will be fixed before release.
- Cycle detection output ordering may differ if Python uses a different traversal order.

## Fixtures

`fixtures/.tickets/` contains a curated corpus:
- `01-basic.md` — minimal done ticket
- `02-with-deps.md` — all optional fields set
- `03-adversarial.md` — quotes in title (tests escaping)
