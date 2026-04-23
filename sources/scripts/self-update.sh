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

echo "==> Updating scripts..."
mkdir -p "$DEST/sources/scripts"
for f in "$SRC"/sources/scripts/*; do
    [ -f "$f" ] && install -m755 "$f" "$DEST/sources/scripts/"
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

# Ensure the scx_loader backend the toolkit now depends on is installed.
# `--needed` keeps this a no-op when the packages are already present.
echo "==> Ensuring scx-scheds + scx-tools are installed..."
if command -v pacman >/dev/null 2>&1; then
    pacman -S --needed --noconfirm scx-scheds scx-tools 2>/dev/null \
        || echo "[WARN] could not install scx-scheds/scx-tools — the scheduler tab will be limited"
fi

# Migrate legacy toolkit-managed scx.service → scx_loader.service.
# Older versions of the toolkit wrote /etc/systemd/system/scx.service from a
# template that looked like:
#   Description=sched-ext BPF CPU Scheduler (<name>)
#   ExecStart=/usr/bin/scx_<name>
# Detect exactly that shape so we don't touch unrelated user-authored units.
LEGACY_UNIT=/etc/systemd/system/scx.service
if [ -f "$LEGACY_UNIT" ] \
    && grep -q '^Description=sched-ext BPF CPU Scheduler' "$LEGACY_UNIT" \
    && grep -qE '^ExecStart=/usr/bin/scx_' "$LEGACY_UNIT"; then
    echo "==> Migrating legacy scx.service → scx_loader.service..."
    LEGACY_SCHED=$(awk -F'/' '/^ExecStart=/ {print $NF; exit}' "$LEGACY_UNIT")
    systemctl disable --now scx.service 2>/dev/null || true
    rm -f "$LEGACY_UNIT"
    systemctl daemon-reload 2>/dev/null || true

    # Seed /etc/scx_loader.toml with the previously-pinned scheduler so the
    # user's choice survives the migration. Don't clobber an existing config.
    if [ -n "$LEGACY_SCHED" ] && [ ! -f /etc/scx_loader.toml ]; then
        printf '# Migrated by CyberXero Toolkit self-update.\ndefault_sched = "%s"\ndefault_mode = "auto"\n' \
            "$LEGACY_SCHED" > /etc/scx_loader.toml
        systemctl enable --now scx_loader.service 2>/dev/null || true
        echo "    preserved scheduler '$LEGACY_SCHED' via /etc/scx_loader.toml"
    fi
fi

echo "==> Refreshing icon cache..."
gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor 2>/dev/null || true

if [ -n "$COMMIT_HASH" ]; then
    echo "==> Recording version..."
    echo "$COMMIT_HASH" > "$DEST/.commit"
fi

echo "==> Update installed successfully."
