#!/usr/bin/env bash
# Parity test harness: runs both Rust tkt and Python tkt against the same
# fixtures and compares output.
#
# Prerequisites:
#   - `tkt` (Rust) on PATH (cargo install or local build)
#   - Python tkt accessible via: python -m tkt (or TKT_PYTHON set to the command)
#
# Usage:
#   tests/parity/run.sh [--verbose]
#
# Exit: 0 = all match, 1 = differences found

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FIXTURES="$SCRIPT_DIR/fixtures"
TKT_RUST="${TKT_RUST:-tkt}"
TKT_PYTHON="${TKT_PYTHON:-python -m tkt}"

VERBOSE="${1:-}"
FAILURES=0
TESTS=0

compare() {
    local cmd_name="$1"
    shift
    local args=("$@")
    
    TESTS=$((TESTS + 1))
    
    local rust_out python_out rust_exit python_exit
    
    rust_out=$($TKT_RUST "${args[@]}" 2>/dev/null) || rust_exit=$?
    rust_exit=${rust_exit:-0}
    
    python_out=$($TKT_PYTHON "${args[@]}" 2>/dev/null) || python_exit=$?
    python_exit=${python_exit:-0}
    
    if [ "$rust_exit" != "$python_exit" ]; then
        echo "FAIL [$cmd_name] exit code: rust=$rust_exit python=$python_exit"
        FAILURES=$((FAILURES + 1))
        return
    fi
    
    if [ "$rust_out" != "$python_out" ]; then
        echo "FAIL [$cmd_name] output differs"
        if [ "$VERBOSE" = "--verbose" ]; then
            echo "  RUST:   $rust_out"
            echo "  PYTHON: $python_out"
        fi
        FAILURES=$((FAILURES + 1))
        return
    fi
    
    echo "PASS [$cmd_name]"
}

# Set up temp repo with fixtures
TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

cp -r "$FIXTURES/.tickets" "$TMPDIR/"
cd "$TMPDIR"
git init -q -b main
git add -A
git commit -qm "fixture"

echo "=== Parity tests against fixture corpus ==="
echo ""

# Read-only commands (safe to compare)
compare "ready" ready
compare "ready --json" ready --json
compare "validate --brief" validate --brief
compare "validate (json)" validate
compare "query" query

echo ""
echo "=== Results: $((TESTS - FAILURES))/$TESTS passed ==="

if [ $FAILURES -gt 0 ]; then
    echo "DIVERGENCES:"
    echo "  Some commands produce different output between Rust and Python."
    echo "  Review each FAIL above. Some may be intentional improvements."
    exit 1
fi

echo "All outputs match."
exit 0
