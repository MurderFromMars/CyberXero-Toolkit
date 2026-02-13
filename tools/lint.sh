#!/usr/bin/env bash
set -euo pipefail

# lint.sh — format Rust crates and GUI XML resources
# Usage: ./lint.sh (can be run from any directory)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." >/dev/null 2>&1 && pwd)"
cd "$REPO_ROOT"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

need_cmd() {
  command -v "$1" >/dev/null 2>&1
}

fmt_rust_crate() {
  local dir="$1"
  if [ ! -f "$dir/Cargo.toml" ]; then
    echo "Skipping $dir: no Cargo.toml found"
    return
  fi
  if ! need_cmd cargo; then
    echo "cargo not found; skipping $dir"
    return
  fi
  echo "Running cargo fmt in $dir..."
  (cd "$dir" && cargo fmt)
}

fmt_xml_file() {
  local f="$1"
  local tmp
  tmp="$(mktemp)"
  trap 'rm -f "$tmp"' RETURN

  echo "  $f"
  if xmllint --format "$f" > "$tmp" 2>/dev/null; then
    mv "$tmp" "$f"
  else
    echo "  xmllint failed for $f — leaving unchanged"
  fi
}

# ---------------------------------------------------------------------------
# Rust formatting
# ---------------------------------------------------------------------------

for dir in gui xero-auth; do
  fmt_rust_crate "$dir"
done

# ---------------------------------------------------------------------------
# XML / UI / SVG formatting
# ---------------------------------------------------------------------------

RES_DIR="gui/resources"

if ! need_cmd xmllint; then
  echo "xmllint not found; skipping XML formatting"
elif [ ! -d "$RES_DIR" ]; then
  echo "Resource directory $RES_DIR not found; skipping XML formatting"
else
  echo "Formatting .ui/.xml/.svg files under $RES_DIR..."
  while IFS= read -r -d '' f; do
    fmt_xml_file "$f"
  done < <(find "$RES_DIR" -type f \( -iname '*.ui' -o -iname '*.xml' -o -iname '*.svg' \) -print0)
fi

echo "Done."
