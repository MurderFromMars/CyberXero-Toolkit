#!/usr/bin/env bash
#
# CyberXero Toolkit — self-update install step
#
# Called from the running toolkit after cloning + building the latest version.
# This script lives in the NEW repo clone, so it always reflects the latest
# install logic — the old binary never has to know how to install the new one.
#
# SRC is derived from this script's own location so we work regardless of
# whatever tmp directory the caller chose to clone into.
#
# Usage: self-update.sh <commit_hash>
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SRC="$(dirname "$(dirname "$SCRIPT_DIR")")"
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

echo "==> Installing symlink, desktop entry, and icon..."
ln -sf "$DEST/cyberxero-toolkit" /usr/bin/cyberxero-toolkit
if [ -f "$SRC/packaging/cyberxero-toolkit.desktop" ]; then
    install -Dm644 "$SRC/packaging/cyberxero-toolkit.desktop" \
        /usr/share/applications/cyberxero-toolkit.desktop
fi
if [ -f "$SRC/gui/resources/icons/scalable/apps/cyberxero-toolkit.png" ]; then
    install -Dm644 "$SRC/gui/resources/icons/scalable/apps/cyberxero-toolkit.png" \
        /usr/share/icons/hicolor/scalable/apps/cyberxero-toolkit.png
fi

# Remove legacy xero-toolkit artifacts from pre-rebrand installs.
echo "==> Cleaning up legacy xero-toolkit artifacts..."
rm -rf /opt/xero-toolkit
rm -f /usr/bin/xero-toolkit \
      /usr/share/applications/xero-toolkit.desktop \
      /usr/share/icons/hicolor/scalable/apps/xero-toolkit.png \
      /etc/xdg/autostart/xero-toolkit.desktop
for home in /home/*; do
    rm -f "$home/.config/autostart/xero-toolkit.desktop" 2>/dev/null || true
done

echo "==> Refreshing icon cache..."
gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor 2>/dev/null || true

if [ -n "$COMMIT_HASH" ]; then
    echo "==> Recording version..."
    echo "$COMMIT_HASH" > "$DEST/.commit"
fi

echo "==> Update installed successfully."
