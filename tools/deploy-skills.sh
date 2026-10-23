#!/usr/bin/env bash
set -euo pipefail

# Deploy tkt's core skills to agent harness scan roots, following grey-matter
# conventions: COPY (not symlink), manifest-tracked, name==dir + global-unique
# lint, and only ever touch skills this installer owns.
#
# tkt maintains its own core skills (skills/<name>/SKILL.md) because they track
# the tool's features and behaviors as they change. grey-matter writes its own
# wrapper around these; this script is the tkt-repo-local deploy path.
#
# Usage:
#   tools/deploy-skills.sh [--dry-run] [--force] [--root DIR]...
#   tools/deploy-skills.sh list
#   tools/deploy-skills.sh uninstall [--dry-run] [--root DIR]...
#
# Flags:
#   --dry-run   print the plan, change nothing
#   --force     overwrite a same-name dir in a root that this installer did not
#               install (default: refuse)
#   --root DIR  install into DIR instead of the default roots (repeatable)
#
# Default roots: every known harness skills root whose parent exists, since each
# harness scans its own root (kiro ~/.kiro/skills, codex ~/.codex/skills, Claude
# ~/.claude/skills, plus the cross-client ~/.agents/skills). On default install
# (no explicit --root), also packages and installs the Antigravity (agy) plugin
# into ~/.gemini/config/plugins/tkt/ (via `agy plugin validate/install` if agy
# is on PATH, or direct copy if ~/.gemini/config/plugins exists).

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SKILLS_DIR="$REPO_ROOT/skills"
MANIFEST_NAME=".installed-by-tkt.json"
PLUGIN_NAME="tkt"

CMD="install"
DRY_RUN=false
FORCE=false
EXPLICIT_ROOTS=false
ROOTS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    install|list|uninstall) CMD="$1" ;;
    --dry-run) DRY_RUN=true ;;
    --force) FORCE=true ;;
    --root) shift; ROOTS+=("$1"); EXPLICIT_ROOTS=true ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
  shift
done

# Default roots: known harness roots whose PARENT dir exists on this machine.
if [[ ${#ROOTS[@]} -eq 0 ]]; then
  for r in "$HOME/.kiro/skills" "$HOME/.codex/skills" "$HOME/.claude/skills" "$HOME/.agents/skills"; do
    [[ -d "$(dirname "$r")" ]] && ROOTS+=("$r")
  done
  [[ ${#ROOTS[@]} -eq 0 ]] && ROOTS=("$HOME/.agents/skills")
fi

# --- Discover skills: every skills/<name>/ with a SKILL.md (flat layout) ---
discover() {
  SKILL_NAMES=()
  for d in "$SKILLS_DIR"/*/; do
    [[ -f "${d}SKILL.md" ]] || continue
    SKILL_NAMES+=("$(basename "$d")")
  done
}

# Read frontmatter name: from a SKILL.md
fm_name() {
  sed -n '/^---/,/^---/p' "$1" | sed -n 's/^name:[[:space:]]*//p' | head -1 | tr -d '"'"'"' '
}

# --- Hard lint: name==dir + globally-unique names + tkt self-sufficiency ---
# Self-sufficiency: tkt must work standalone for an adopter who has tkt but NOT
# grey-matter. A distributed skill therefore must not reference a non-tkt skill by
# name (frontier-work, read-handoff, to-tickets, ...) — those won't exist for a
# tkt-only user. Referencing sibling tkt-* skills and `tkt` commands is fine.
FOREIGN_SKILLS='frontier-work|read-handoff|to-tickets|spec-driven-development|reconcile-tickets|wrap-session|compass|implement-spec'
lint() {
  local errors=0
  declare -A seen
  for name in "${SKILL_NAMES[@]}"; do
    local fm; fm="$(fm_name "$SKILLS_DIR/$name/SKILL.md")"
    if [[ -n "$fm" && "$fm" != "$name" ]]; then
      echo "  lint: $name/SKILL.md declares name \"$fm\" but its folder is \"$name\"" >&2
      errors=$((errors + 1))
    fi
    local foreign; foreign="$(grep -oE "\b($FOREIGN_SKILLS)\b" "$SKILLS_DIR/$name/SKILL.md" | sort -u | tr '\n' ' ')"
    if [[ -n "${foreign// }" ]]; then
      echo "  lint: $name/SKILL.md references non-tkt skill(s): ${foreign}(tkt skills must be self-sufficient — use tkt commands or sibling tkt-* skills)" >&2
      errors=$((errors + 1))
    fi
    if [[ -n "${seen[$name]:-}" ]]; then
      echo "  lint: duplicate skill name \"$name\"" >&2
      errors=$((errors + 1))
    fi
    seen[$name]=1
  done
  return $errors
}

read_manifest() {  # echoes space-separated owned names
  local mf="$1/$MANIFEST_NAME"
  [[ -f "$mf" ]] || return 0
  sed -n 's/.*"installed":[[:space:]]*\[\(.*\)\].*/\1/p' "$mf" | tr -d '"' | tr ',' ' '
}

write_manifest() {
  local root="$1"; shift
  $DRY_RUN && return 0
  local joined=""
  for n in "$@"; do joined+="\"$n\", "; done
  joined="${joined%, }"
  cat > "$root/$MANIFEST_NAME" <<EOF
{
  "installer": "tkt/tools/deploy-skills.sh",
  "repo": "$REPO_ROOT",
  "mode": "copy",
  "installedAt": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "installed": [$joined]
}
EOF
}

do_install() {
  discover
  [[ ${#SKILL_NAMES[@]} -gt 0 ]] || { echo "no skills under $SKILLS_DIR/<name>/" >&2; exit 1; }
  if ! lint; then echo "install aborted: lint failed" >&2; exit 1; fi

  local refused_total=0
  for root in "${ROOTS[@]}"; do
    echo ""
    echo "root: $root  ($($DRY_RUN && echo dry-run || echo copy))"
    $DRY_RUN || mkdir -p "$root"
    local owned=" $(read_manifest "$root") "

    # Prune: names we installed before but are no longer in source.
    for prev in $(read_manifest "$root"); do
      local still=false
      for name in "${SKILL_NAMES[@]}"; do [[ "$prev" == "$name" ]] && still=true; done
      if ! $still; then
        echo "  prune   $prev"
        $DRY_RUN || rm -rf "${root:?}/$prev"
      fi
    done

    for name in "${SKILL_NAMES[@]}"; do
      local dest="$root/$name"
      if [[ -e "$dest" && "$owned" != *" $name "* ]] && ! $FORCE; then
        echo "  refused $name (exists, not installed by us; --force to override)"
        refused_total=$((refused_total + 1))
        continue
      fi
      if $DRY_RUN; then
        echo "  $([[ -e "$dest" ]] && echo replace || echo create) $name"
        continue
      fi
      rm -rf "$dest"
      cp -R "$SKILLS_DIR/$name" "$dest"
      echo "  copied  $name"
    done

    write_manifest "$root" "${SKILL_NAMES[@]}"
  done

  # --- Antigravity (agy) plugin ---
  # On agy, global skill discovery is plugin-driven (~/.gemini/config/plugins/<name>/).
  # Package plugin.json + skills/ and install when running with default roots.
  if ! $EXPLICIT_ROOTS; then
    deploy_agy_plugin
  fi

  echo ""
  echo "$($DRY_RUN && echo '(dry-run) would install' || echo installed) ${#SKILL_NAMES[@]} skills into ${#ROOTS[@]} root(s)."
  [[ $refused_total -gt 0 ]] && echo "$refused_total refused (foreign dirs); re-run with --force to overwrite."
  return 0
}

deploy_agy_plugin() {
  local plugins_root="$HOME/.gemini/config/plugins"
  local plugin_dest="$plugins_root/$PLUGIN_NAME"
  local has_agy=false
  command -v agy >/dev/null 2>&1 && has_agy=true

  if ! $has_agy && [[ ! -d "$plugins_root" ]]; then
    return 0
  fi

  echo ""
  echo "agy plugin: $plugin_dest  ($($DRY_RUN && echo dry-run || echo plugin))"
  if $DRY_RUN; then
    echo "  would package $REPO_ROOT/plugin.json + ${#SKILL_NAMES[@]} skills -> $plugin_dest"
    return 0
  fi

  local stage_dir
  stage_dir="$(mktemp -d "${TMPDIR:-/tmp}/tkt-agy-plugin.XXXXXX")"
  mkdir -p "$stage_dir/skills"
  cp "$REPO_ROOT/plugin.json" "$stage_dir/plugin.json"
  for name in "${SKILL_NAMES[@]}"; do
    cp -R "$SKILLS_DIR/$name" "$stage_dir/skills/$name"
  done

  if $has_agy; then
    echo "  validating: agy plugin validate --strict"
    agy plugin validate "$stage_dir" --strict >/dev/null
    echo "  installing: agy plugin install"
    agy plugin install "$stage_dir"
  else
    mkdir -p "$plugins_root"
    rm -rf "$plugin_dest"
    cp -R "$stage_dir" "$plugin_dest"
    echo "  copied plugin bundle (${#SKILL_NAMES[@]} skills) -> $plugin_dest"
  fi
  rm -rf "$stage_dir"
}

do_list() {
  for root in "${ROOTS[@]}"; do
    echo ""
    echo "root: $root"
    local names; names="$(read_manifest "$root")"
    if [[ -z "${names// }" ]]; then echo "  (nothing installed by tkt)"; continue; fi
    for n in $names; do echo "  $n"; done
  done
  if ! $EXPLICIT_ROOTS && [[ -d "$HOME/.gemini/config/plugins/$PLUGIN_NAME" ]]; then
    echo ""
    echo "agy plugin: $HOME/.gemini/config/plugins/$PLUGIN_NAME"
    for d in "$HOME/.gemini/config/plugins/$PLUGIN_NAME/skills"/*/; do
      [[ -f "${d}SKILL.md" ]] && echo "  $(basename "$d")"
    done
  fi
}

do_uninstall() {
  for root in "${ROOTS[@]}"; do
    echo ""
    echo "root: $root  ($($DRY_RUN && echo dry-run || echo uninstall))"
    local names; names="$(read_manifest "$root")"
    if [[ -z "${names// }" ]]; then echo "  (nothing to uninstall)"; continue; fi
    for n in $names; do
      echo "  remove  $n"
      $DRY_RUN || rm -rf "${root:?}/$n"
    done
    $DRY_RUN || rm -f "$root/$MANIFEST_NAME"
  done
  if ! $EXPLICIT_ROOTS && [[ -d "$HOME/.gemini/config/plugins/$PLUGIN_NAME" ]]; then
    echo ""
    echo "agy plugin: $HOME/.gemini/config/plugins/$PLUGIN_NAME  ($($DRY_RUN && echo dry-run || echo uninstall))"
    echo "  remove  $PLUGIN_NAME"
    $DRY_RUN || rm -rf "$HOME/.gemini/config/plugins/$PLUGIN_NAME"
  fi
}

case "$CMD" in
  install) do_install ;;
  list) do_list ;;
  uninstall) do_uninstall ;;
esac
