#!/usr/bin/env bash
# Build the release binary and install tkt to a stable, user-controlled root.
#
# Why not `cargo install --path .` alone: on machines where a version manager
# (mise, rustup shims) owns CARGO_HOME, the binary lands inside a
# toolchain-managed directory that upgrades and prunes can wipe. This script
# installs to <root>/bin (default ~/.local/bin), a location the user owns and
# keeps on PATH.
#
# Usage: bash tools/install-local.sh [--root DIR]
set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)" || exit 2
root="${HOME}/.local"
while [ $# -gt 0 ]; do
  case "$1" in
    --root) root="${2:?--root needs a directory}"; shift 2 ;;
    --root=*) root="${1#--root=}"; shift ;;
    -h|--help) sed -n '2,10p' "${BASH_SOURCE[0]}"; exit 0 ;;
    *) echo "install-local: unknown argument: $1" >&2; exit 2 ;;
  esac
done

if ! command -v cargo >/dev/null 2>&1; then
  echo "install-local: cargo not on PATH; install a toolchain first (rustup default <ver>)" >&2
  exit 2
fi

bin_name="tkt"
case "${OSTYPE:-}" in
  msys*|cygwin*|win32) bin_name="tkt.exe" ;;
esac
# Windows Git Bash can report a linux-ish OSTYPE under some wrappers; the target dir decides.
[ -f "$repo_root/target/release/tkt.exe" ] && bin_name="tkt.exe"

dest_dir="$root/bin"
dest="$dest_dir/$bin_name"

echo "install-local: building release (target/ cache reused when fresh)..."
(cd "$repo_root" && cargo build --release)

src="$repo_root/target/release/$bin_name"
[ -f "$src" ] || { echo "install-local: build produced no $src" >&2; exit 1; }

mkdir -p "$dest_dir"
cp -f "$src" "$dest"
echo "install-local: installed $dest"

# Freshness gate: the installed binary's embedded git hash must equal HEAD.
head_hash="$(git -C "$repo_root" rev-parse --short=7 HEAD)"
version="$("$dest" --version 2>&1)" || { echo "install-local: installed binary fails to run: $version" >&2; exit 1; }
embedded="$(printf '%s' "$version" | grep -oiE '\([0-9a-f]{7,40}(-dirty)?\)' | tr -d '()' | head -1)"
if [ -z "$embedded" ]; then
  echo "install-local: installed --version has no embedded git hash: $version" >&2
  exit 1
fi
if [ "${embedded%-dirty}" != "$head_hash" ]; then
  echo "install-local: STALE install: installed $embedded, HEAD is $head_hash" >&2
  exit 1
fi
echo "install-local: fresh: $version"

# PATH guidance: warn when the destination is not where `tkt` resolves.
resolved="$(command -v tkt 2>/dev/null || true)"
if [ -z "$resolved" ]; then
  echo "install-local: NOTE: no tkt on PATH; add to PATH: $dest_dir"
elif [ "$(cd -- "$(dirname -- "$resolved")" && pwd)" != "$(cd -- "$dest_dir" && pwd)" ]; then
  echo "install-local: NOTE: PATH resolves tkt to $resolved, not $dest"
  echo "install-local: check PATH order or remove the shadowing copy"
fi
