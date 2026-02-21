#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: tools/bump-version.sh <major|minor|subminor|sync>

Actions:
  major     Bump X in X.Y.Z, reset Y and Z to 0
  minor     Bump Y in X.Y.Z, reset Z to 0
  subminor  Bump Z in X.Y.Z
  sync      Keep version as-is from PKGBUILD and sync Cargo.toml to it

Notes:
  - Version format is strictly one digit per component: X.Y.Z
  - Each component must be in range 0..9
  - Bumps that would exceed 9 are rejected
EOF
}

if [[ $# -ne 1 ]]; then
  usage
  exit 1
fi

action="$1"

case "$action" in
major|minor|subminor|sync) ;;
*)
  usage
  exit 1
  ;;
esac

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." >/dev/null 2>&1 && pwd)"
CARGO_FILE="$REPO_ROOT/Cargo.toml"
PKGBUILD_FILE="$REPO_ROOT/packaging/PKGBUILD"

extract_cargo_version() {
  sed -nE 's/^version = "([0-9]+\.[0-9]+\.[0-9]+)"/\1/p' "$CARGO_FILE" | head -n1
}

extract_pkgbuild_version() {
  sed -nE 's/^pkgver=([0-9]+\.[0-9]+\.[0-9]+)$/\1/p' "$PKGBUILD_FILE" | head -n1
}

is_single_digit_triplet() {
  [[ "$1" =~ ^[0-9]\.[0-9]\.[0-9]$ ]]
}

set_versions() {
  local new_version="$1"
  local tmp_file

  tmp_file="$(mktemp)"
  awk -v v="$new_version" '
    BEGIN { in_workspace_package = 0; done = 0 }
    {
      if ($0 ~ /^\[workspace.package\]$/) {
        in_workspace_package = 1
        print
        next
      }

      if (in_workspace_package && !done && $0 ~ /^version = "/) {
        print "version = \"" v "\""
        done = 1
        next
      }

      print
    }
    END {
      if (!done) {
        exit 2
      }
    }
  ' "$CARGO_FILE" > "$tmp_file"
  cat "$tmp_file" > "$CARGO_FILE"
  rm -f "$tmp_file"

  tmp_file="$(mktemp)"
  awk -v v="$new_version" '
    BEGIN { done = 0 }
    {
      if (!done && $0 ~ /^pkgver=/) {
        print "pkgver=" v
        done = 1
        next
      }
      print
    }
    END {
      if (!done) {
        exit 2
      }
    }
  ' "$PKGBUILD_FILE" > "$tmp_file"
  cat "$tmp_file" > "$PKGBUILD_FILE"
  rm -f "$tmp_file"
}

cargo_version="$(extract_cargo_version)"
pkgbuild_version="$(extract_pkgbuild_version)"

if [[ -z "$cargo_version" ]]; then
  echo "Error: unable to read workspace version from $CARGO_FILE" >&2
  exit 1
fi

if [[ -z "$pkgbuild_version" ]]; then
  echo "Error: unable to read pkgver from $PKGBUILD_FILE" >&2
  exit 1
fi

if ! is_single_digit_triplet "$cargo_version"; then
  echo "Error: Cargo version '$cargo_version' must be one-digit triplet (X.Y.Z with 0..9)." >&2
  exit 1
fi

if ! is_single_digit_triplet "$pkgbuild_version"; then
  echo "Error: PKGBUILD version '$pkgbuild_version' must be one-digit triplet (X.Y.Z with 0..9)." >&2
  exit 1
fi

if [[ "$cargo_version" != "$pkgbuild_version" ]]; then
  echo "Notice: Cargo version ($cargo_version) differs from PKGBUILD ($pkgbuild_version); using PKGBUILD as source." >&2
fi

new_version="$pkgbuild_version"

if [[ "$action" != "sync" ]]; then
  IFS='.' read -r major minor subminor <<< "$pkgbuild_version"
  case "$action" in
    major)
      if (( major >= 9 )); then
        echo "Error: cannot bump major beyond 9." >&2
        exit 1
      fi
      new_version="$((major + 1)).0.0"
      ;;
    minor)
      if (( minor >= 9 )); then
        echo "Error: cannot bump minor beyond 9." >&2
        exit 1
      fi
      new_version="${major}.$((minor + 1)).0"
      ;;
    subminor)
      if (( subminor >= 9 )); then
        echo "Error: cannot bump subminor beyond 9." >&2
        exit 1
      fi
      new_version="${major}.${minor}.$((subminor + 1))"
      ;;
  esac
fi

set_versions "$new_version"

echo "Updated versions:"
echo "  PKGBUILD: $pkgbuild_version -> $new_version"
echo "  Cargo.toml: $cargo_version -> $new_version"
