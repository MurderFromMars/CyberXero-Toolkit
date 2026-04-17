#!/usr/bin/env bash
#
# CyberXero Toolkit — self-update install step
#
# Called from the running toolkit after cloning + building the latest version.
# This script lives in the NEW repo clone, so it always reflects the latest
# install logic — the old binary never has to know how to install the new one.
#
# Usage: self-update.sh <commit_hash>
#
set -euo pipefail

SRC="/tmp/cyberxero-toolkit-update"
DEST="/opt/cyberxero-toolkit"
COMMIT_HASH="${1:-}"

die() { echo "[ERROR] $1" >&2; exit 1; }

[ -d "$SRC" ] || die "Source directory $SRC not found"
[ -d "$DEST" ] || mkdir -p "$DEST"

echo "==> Installing binaries..."
for bin in cyberxero-toolkit cyberxero-authd cyberxero-auth; do
    [ -f "$SRC/target/release/$bin" ] || die "Binary not found: $bin"
    install -Dm755 "$SRC/target/release/$bin" "$DEST/$bin"
done

echo "==> Updating scripts and systemd units..."
mkdir -p "$DEST/sources/scripts" "$DEST/sources/systemd"
for f in "$SRC"/sources/scripts/*; do
    [ -f "$f" ] && install -m755 "$f" "$DEST/sources/scripts/"
done
for f in "$SRC"/sources/systemd/*; do
    [ -f "$f" ] && install -m644 "$f" "$DEST/sources/systemd/"
done

echo "==> Updating extra scripts..."
if [ -d "$SRC/extra-scripts/usr/local/bin" ]; then
    for f in "$SRC"/extra-scripts/usr/local/bin/*; do
        [ -f "$f" ] && install -m755 "$f" /usr/local/bin/
    done
fi

if [ -n "$COMMIT_HASH" ]; then
    echo "==> Recording version..."
    echo "$COMMIT_HASH" > "$DEST/.commit"
fi

echo "==> Update installed successfully."
