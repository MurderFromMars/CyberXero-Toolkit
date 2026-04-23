#!/bin/bash
#
# CyberXero Toolkit - Installer
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

# ── AUR Helper Check ─────────────────────────────────────────────────────────
install_aur_helper() {
    local helper="$1"
    local repo_url=""

    case "$helper" in
        paru) repo_url="https://aur.archlinux.org/paru.git" ;;
        yay)  repo_url="https://aur.archlinux.org/yay.git"  ;;
    esac

    print_status "Installing $helper from the AUR..."

    # Ensure base-devel and git are available (required for makepkg)
    if ! pacman -Qi base-devel &> /dev/null || ! command -v git &> /dev/null; then
        print_status "Installing required build dependencies (base-devel, git)..."
        sudo pacman -S --needed --noconfirm base-devel git || die "Failed to install build dependencies"
    fi

    local tmp_dir
    tmp_dir="$(mktemp -d)" || die "Failed to create temp directory"
    trap "rm -rf '$tmp_dir'" RETURN

    git clone "$repo_url" "$tmp_dir/$helper" || die "Failed to clone $helper repository"
    (cd "$tmp_dir/$helper" && makepkg -si --noconfirm) || die "Failed to build/install $helper"

    print_success "$helper installed successfully"
}

check_aur_helper() {
    if command -v paru &> /dev/null; then
        print_success "AUR helper found: paru"
        return
    elif command -v yay &> /dev/null; then
        print_success "AUR helper found: yay"
        return
    fi

    print_warning "No AUR helper (paru or yay) was found on this system."
    echo ""
    echo -e "  ${BOLD}1)${NC} Install paru  ${CYAN}(Rust-based, recommended)${NC}"
    echo -e "  ${BOLD}2)${NC} Install yay   ${CYAN}(Go-based, widely used)${NC}"
    echo -e "  ${BOLD}3)${NC} Skip          ${YELLOW}(continue without an AUR helper)${NC}"
    echo ""

    while true; do
        read -rp "Choose an option [1/2/3]: " aur_choice
        case "$aur_choice" in
            1) install_aur_helper paru; break ;;
            2) install_aur_helper yay;  break ;;
            3) print_warning "Skipping AUR helper installation."; break ;;
            *) print_warning "Invalid choice, please enter 1, 2, or 3." ;;
        esac
    done
}

check_aur_helper
# ─────────────────────────────────────────────────────────────────────────────

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
    "scx-tools"
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

# Build the project
print_status "Building CyberXero Toolkit (this may take a few minutes)..."
cd "$SCRIPT_DIR"

cargo clean

if ! cargo build --release; then
    die "Build failed. Check the error messages above."
fi

print_success "Build completed successfully"

# Verify binaries exist
for binary in "cyberxero-toolkit" "cyberxero-authd" "cyberxero-auth"; do
    if [ ! -f "target/release/$binary" ]; then
        die "Binary not found: target/release/$binary"
    fi
done

# Install
print_status "Installing to /opt/cyberxero-toolkit..."

# Create directories
sudo mkdir -p /opt/cyberxero-toolkit || die "Failed to create /opt/cyberxero-toolkit"
sudo mkdir -p /opt/cyberxero-toolkit/sources/scripts

# Install binaries
print_status "Installing binaries..."
sudo install -Dm755 "target/release/cyberxero-toolkit" "/opt/cyberxero-toolkit/cyberxero-toolkit" || die "Failed to install cyberxero-toolkit"
sudo install -Dm755 "target/release/cyberxero-authd" "/opt/cyberxero-toolkit/cyberxero-authd" || die "Failed to install cyberxero-authd"
sudo install -Dm755 "target/release/cyberxero-auth" "/opt/cyberxero-toolkit/cyberxero-auth" || die "Failed to install cyberxero-auth"

# Install sources
print_status "Installing scripts..."
sudo install -m755 sources/scripts/* "/opt/cyberxero-toolkit/sources/scripts/" || die "Failed to install scripts"

# Create symlink in /usr/bin
print_status "Creating symlink..."
sudo ln -sf "/opt/cyberxero-toolkit/cyberxero-toolkit" "/usr/bin/cyberxero-toolkit" || die "Failed to create symlink"

# Install desktop file
print_status "Installing desktop file..."
sudo install -Dm644 "packaging/cyberxero-toolkit.desktop" \
    "/usr/share/applications/cyberxero-toolkit.desktop" || die "Failed to install desktop file"

# Install icon
print_status "Installing icon..."
sudo install -Dm644 "gui/resources/icons/scalable/apps/cyberxero-toolkit.png" \
    "/usr/share/icons/hicolor/scalable/apps/cyberxero-toolkit.png" || die "Failed to install icon"

# Update icon cache
print_status "Updating icon cache..."
sudo gtk-update-icon-cache -q -t -f /usr/share/icons/hicolor 2>/dev/null || true

# Store current commit hash for update checking
if command -v git &> /dev/null && git -C "$SCRIPT_DIR" rev-parse HEAD &> /dev/null; then
    COMMIT_HASH=$(git -C "$SCRIPT_DIR" rev-parse HEAD)
    echo "$COMMIT_HASH" | sudo tee /opt/cyberxero-toolkit/.commit > /dev/null
    print_success "Stored commit hash: ${COMMIT_HASH:0:12}"
fi

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
echo -e "  ${BOLD}cyberxero-toolkit${NC}"
echo ""

# Optional: Launch it
read -p "Launch CyberXero Toolkit now? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    cyberxero-toolkit &
fi
