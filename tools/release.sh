#!/usr/bin/env bash
# Release script for tkt — fallback for environments without mise
# Usage: tools/release.sh [patch|minor|major] [--execute]
#
# Dry-run by default. Pass --execute to actually cut the release.
set -euo pipefail

LEVEL="${1:-patch}"
shift || true

echo "=== Pre-release gates ==="
cargo fmt --check || { echo "❌ fmt check failed"; exit 1; }
cargo clippy --all-targets -- -D warnings || { echo "❌ clippy failed"; exit 1; }
cargo test || { echo "❌ tests failed"; exit 1; }
echo "✓ All gates passed"
echo ""

echo "=== Running cargo release $LEVEL $* ==="
cargo release "$LEVEL" "$@"
