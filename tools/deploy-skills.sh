#!/usr/bin/env bash
set -euo pipefail

# Deploy tkt skills and steering to AI agent tool paths.
# Usage: tools/deploy-skills.sh [--dry-run]
#
# Skills: symlinked (live updates during development)
# Steering: copied (steering is body-only, always loaded)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SKILL_SRC="$REPO_ROOT/skills/tkt"
STEERING_SRC="$REPO_ROOT/skills/steering"

DRY_RUN=false
[[ "${1:-}" == "--dry-run" ]] && DRY_RUN=true

deployed=0
skipped=0

# --- Deploy skills (symlink) ---

declare -A SKILL_TARGETS=(
  [kiro]="$HOME/.kiro/skills/tkt"
  [claude]="$HOME/.claude/skills/tkt"
  [codex]="$HOME/.codex/skills/tkt"
)

echo "Skills:"
for harness in "${!SKILL_TARGETS[@]}"; do
  target="${SKILL_TARGETS[$harness]}"
  parent="$(dirname "$target")"
  harness_root="$(dirname "$parent")"

  if [[ ! -d "$harness_root" ]]; then
    echo "  skip  $harness (${harness_root} not found)"
    skipped=$((skipped + 1))
    continue
  fi

  if $DRY_RUN; then
    echo "  would symlink → $target"
    deployed=$((deployed + 1))
    continue
  fi

  mkdir -p "$parent"
  rm -rf "$target"
  ln -sf "$SKILL_SRC" "$target"
  echo "  ✓ $harness → $target"
  deployed=$((deployed + 1))
done

# --- Deploy steering (copy) ---

if [[ -d "$STEERING_SRC" ]]; then
  echo ""
  echo "Steering:"

  # Only deploy to kiro (steering is a kiro-cli concept)
  steering_dest="$HOME/.kiro/steering"

  if [[ ! -d "$HOME/.kiro" ]]; then
    echo "  skip  steering (~/.kiro not found)"
    skipped=$((skipped + 1))
  else
    mkdir -p "$steering_dest"

    for src_file in "$STEERING_SRC"/*.md; do
      [[ -f "$src_file" ]] || continue
      name="$(basename "$src_file")"

      if $DRY_RUN; then
        echo "  would copy → $steering_dest/$name"
        deployed=$((deployed + 1))
        continue
      fi

      cp "$src_file" "$steering_dest/$name"
      echo "  ✓ $name → $steering_dest/$name"
      deployed=$((deployed + 1))
    done
  fi
fi

echo ""
echo "Deployed: $deployed, Skipped: $skipped"
