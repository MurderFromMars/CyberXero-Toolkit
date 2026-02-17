#!/bin/bash
#
# Xero Toolkit Open - Installer
# Builds from source and installs for any Arch-based distro
#

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

print_status() { echo -e "${CYAN}[*]${NC} $1"; }
print_success() { echo -e "${GREEN}[✓]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[!]${NC} $1"; }
print_error() { echo -e "${RED}[✗]${NC} $1"; }

die() {
    print_error "$1"
    exit 1
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo ""
echo -e "${CYAN}╔═══════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║           CyberXero Toolkit                           ║${NC}"
echo -e "${CYAN}║          The Open Xerolinux Toolkit!                  ║${NC}"
echo -e "${CYAN}╚═══════════════════════════════════════════════════════╝${NC}"
echo ""

# Check if running as root
if [[ $EUID -eq 0 ]]; then
    die "Do not run this script as root. It will ask for sudo when needed."
fi

# Check for Arch-based system
if ! command -v pacman &> /dev/null; then
    die "This installer requires an Arch-based distribution with pacman."
fi

print_success "Detected Arch-based system"

# Check dependencies
print_status "Checking build dependencies..."

DEPS=(
    "rust"
    "cargo" 
    "pkgconf"
    "gtk4"
    "glib2"
    "libadwaita"
    "vte4"
    "flatpak"
    "polkit"
    "base-devel"
    "scx-scheds"
)

MISSING_DEPS=()

for dep in "${DEPS[@]}"; do
    if ! pacman -Qi "$dep" &> /dev/null; then
        MISSING_DEPS+=("$dep")
    fi
done

if [ ${#MISSING_DEPS[@]} -ne 0 ]; then
    print_warning "Missing dependencies: ${MISSING_DEPS[*]}"
    print_status "Installing missing dependencies..."
    if ! sudo pacman -S --needed --noconfirm "${MISSING_DEPS[@]}"; then
        die "Failed to install dependencies"
    fi
fi

print_success "Dependencies satisfied"

# Patch the XeroLinux check
print_status "Patching XeroLinux distribution check..."

SYSTEM_CHECK_FILE="$SCRIPT_DIR/gui/src/core/system_check.rs"

if [ ! -f "$SYSTEM_CHECK_FILE" ]; then
    die "Could not find system_check.rs at $SYSTEM_CHECK_FILE"
fi

# Check if already patched
if grep -q "// PATCHED: Removed XeroLinux check" "$SYSTEM_CHECK_FILE"; then
    print_success "Already patched"
else
    # Create backup
    cp "$SYSTEM_CHECK_FILE" "$SYSTEM_CHECK_FILE.bak"
    
    # Replace the check_system_requirements function to skip XeroLinux check
    if sed -i 's/if !check_xerolinux_distribution() {/if false \&\& !check_xerolinux_distribution() { \/\/ PATCHED: Removed XeroLinux check/g' "$SYSTEM_CHECK_FILE"; then
        print_success "Patched system_check.rs"
    else
        die "Failed to patch system_check.rs"
    fi
fi

# Build the project
print_status "Building Xero Toolkit (this may take a few minutes)..."
cd "$SCRIPT_DIR"

if ! cargo build --release; then
    die "Build failed. Check the error messages above."
fi

print_success "Build completed successfully"

# Verify binaries exist
for binary in "xero-toolkit" "xero-authd" "xero-auth"; do
    if [ ! -f "target/release/$binary" ]; then
        die "Binary not found: target/release/$binary"
    fi
done

# Install
print_status "Installing to /opt/xero-toolkit..."

# Create directories
sudo mkdir -p /opt/xero-toolkit || die "Failed to create /opt/xero-toolkit"
sudo mkdir -p /opt/xero-toolkit/sources/scripts
sudo mkdir -p /opt/xero-toolkit/sources/systemd

# Install binaries
print_status "Installing binaries..."
sudo install -Dm755 "target/release/xero-toolkit" "/opt/xero-toolkit/xero-toolkit" || die "Failed to install xero-toolkit"
sudo install -Dm755 "target/release/xero-authd" "/opt/xero-toolkit/xero-authd" || die "Failed to install xero-authd"
sudo install -Dm755 "target/release/xero-auth" "/opt/xero-toolkit/xero-auth" || die "Failed to install xero-auth"

# Install sources
print_status "Installing scripts and systemd units..."
sudo install -m755 sources/scripts/* "/opt/xero-toolkit/sources/scripts/" || die "Failed to install scripts"
sudo install -m644 sources/systemd/* "/opt/xero-toolkit/sources/systemd/" || die "Failed to install systemd units"

# Create symlink in /usr/bin
print_status "Creating symlink..."
sudo ln -sf "/opt/xero-toolkit/xero-toolkit" "/usr/bin/xero-toolkit" || die "Failed to create symlink"

# Install desktop file
print_status "Installing desktop file..."
sudo install -Dm644 "packaging/xero-toolkit.desktop" \
    "/usr/share/applications/xero-toolkit.desktop" || die "Failed to install desktop file"

# Install icon
print_status "Installing icon..."
sudo install -Dm644 "gui/resources/icons/scalable/apps/xero-toolkit.png" \
    "/usr/share/icons/hicolor/scalable/apps/xero-toolkit.png" || die "Failed to install icon"

# Update icon cache
print_status "Updating icon cache..."
sudo gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor 2>/dev/null || true

# Install extra scripts
EXTRA_SCRIPTS_DIR="$SCRIPT_DIR/extra-scripts/usr/local/bin"

if [[ -d "$EXTRA_SCRIPTS_DIR" ]]; then
    print_status "Installing extra scripts to /usr/local/bin..."

    extra_installed=0
    extra_failed=0

    for script in "$EXTRA_SCRIPTS_DIR"/*; do
        [[ -f "$script" ]] || continue
        name="$(basename "$script")"
        if sudo install -Dm755 "$script" "/usr/local/bin/$name"; then
            ((extra_installed++))
        else
            print_warning "Failed to install: $name"
            ((extra_failed++))
        fi
    done

    if [[ $extra_failed -eq 0 ]]; then
        print_success "Installed $extra_installed extra scripts"
    else
        print_warning "Installed $extra_installed extra scripts ($extra_failed failed)"
    fi
else
    print_warning "Extra scripts directory not found, skipping"
fi

print_success "Installation complete!"

echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  CyberXero Toolkit installed successfully!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════${NC}"
echo ""
echo "You can now launch it from your application menu or run:"
echo -e "  ${BOLD}xero-toolkit${NC}"
echo ""

# Optional: Launch it
read -p "Launch Xero Toolkit now? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    xero-toolkit &
fi
