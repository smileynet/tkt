#!/usr/bin/env bash
# Compatibility entry point; deployment logic is vendored from grey-matter/scripts/deploy.mjs.
# Usage: bash tools/deploy-skills.sh [install|list|uninstall|check|audit] [options]
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec node "$SCRIPT_DIR/deploy.mjs" "$@"
