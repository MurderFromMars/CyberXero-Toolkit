#!/bin/bash
#
# Xero Toolkit Open - Uninstaller
#

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m'

print_status() { echo -e "${CYAN}[*]${NC} $1"; }
print_success() { echo -e "${GREEN}[✓]${NC} $1"; }

echo ""
echo -e "${RED}Uninstalling Xero Toolkit Open...${NC}"
echo ""

# Remove binaries and data
print_status "Removing /opt/xero-toolkit..."
sudo rm -rf /opt/xero-toolkit

# Remove symlink
print_status "Removing symlink..."
sudo rm -f /usr/bin/xero-toolkit

# Remove desktop file
print_status "Removing desktop file..."
sudo rm -f /usr/share/applications/xero-toolkit.desktop

# Remove icon
print_status "Removing icon..."
sudo rm -f /usr/share/icons/hicolor/scalable/apps/xero-toolkit.png

# Remove extra scripts
EXTRA_SCRIPTS=(gcm getcider keyfix opr-drv pacup pmpd rddav rpipe upd xpm)
print_status "Removing extra scripts..."
for name in "${EXTRA_SCRIPTS[@]}"; do
    sudo rm -f "/usr/local/bin/$name"
done

# Update icon cache
print_status "Updating icon cache..."
sudo gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor 2>/dev/null || true

# Remove user autostart if exists
if [ -f "$HOME/.config/autostart/xero-toolkit.desktop" ]; then
    print_status "Removing user autostart..."
    rm -f "$HOME/.config/autostart/xero-toolkit.desktop"
fi

print_success "Xero Toolkit Open has been uninstalled"
echo ""
