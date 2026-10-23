#!/usr/bin/env bash
# Local release preparation/verification; defaults remain read-only. Never publishes.
# Usage: bash release.sh prepare --repo DIR --version X.Y.Z --out DIR [--execute]
#        bash release.sh verify --repo DIR --sha SHA --artifacts DIR --out DIR [--execute]
#        bash release.sh verify-product --repo DIR --sha SHA --out DIR [--execute]
#        bash release.sh verify-native --artifacts DIR --out DIR [--execute]
set -euo pipefail

MODE="${1:-}"
case "$MODE" in
  prepare|verify|verify-product|verify-native) shift ;;
  *) echo "Usage: release.sh {prepare|verify|verify-product|verify-native} [explicit options]; legacy publication is disabled" >&2; exit 2 ;;
esac
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
while true; do
  if [[ -f "$ROOT/scripts/release.mjs" ]]; then TOOLS="$ROOT/scripts"; break; fi
  if [[ -f "$ROOT/tools/release.mjs" ]]; then TOOLS="$ROOT/tools"; break; fi
  PARENT="$(dirname "$ROOT")"
  if [[ "$PARENT" == "$ROOT" ]]; then echo "Shared release tools missing; use a complete source/package checkout" >&2; exit 1; fi
  ROOT="$PARENT"
done
case "$MODE" in
  prepare) exec node "$TOOLS/release.mjs" prepare "$@" ;;
  verify) exec node "$TOOLS/release-verify.mjs" "$@" ;;
  verify-product) exec node "$TOOLS/release-product.mjs" "$@" ;;
  verify-native) exec node "$TOOLS/release-native.mjs" "$@" ;;
esac
