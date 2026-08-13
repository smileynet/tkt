#!/usr/bin/env bash
set -euo pipefail

# Deploy tkt skill to AI agent tool paths.
# Usage: tools/deploy-skills.sh [--dry-run]

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SKILL_SRC="$REPO_ROOT/skills/tkt"

DRY_RUN=false
[[ "${1:-}" == "--dry-run" ]] && DRY_RUN=true

# Target paths (one per harness)
declare -A TARGETS=(
  [kiro]="$HOME/.kiro/skills/tkt"
  [claude]="$HOME/.claude/skills/tkt"
  [codex]="$HOME/.codex/skills/tkt"
)

deployed=0
skipped=0

for harness in "${!TARGETS[@]}"; do
  target="${TARGETS[$harness]}"
  parent="$(dirname "$target")"

  # Only deploy if the harness directory exists (don't create ~/.claude/ for someone who doesn't use it)
  harness_root="$(dirname "$parent")"
  if [[ ! -d "$harness_root" ]]; then
    echo "  skip  $harness (${harness_root} not found)"
    skipped=$((skipped + 1))
    continue
  fi

  if $DRY_RUN; then
    echo "  would deploy → $target"
    deployed=$((deployed + 1))
    continue
  fi

  mkdir -p "$parent"
  # Remove existing (could be stale symlink or old copy)
  rm -rf "$target"
  # Symlink for live updates during development
  ln -sf "$SKILL_SRC" "$target"
  echo "  ✓ $harness → $target"
  deployed=$((deployed + 1))
done

echo ""
echo "Deployed: $deployed, Skipped: $skipped"
