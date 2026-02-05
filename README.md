# 🛠️ CyberXero Toolkit

A GTK4 GUI application for managing system tools, configurations, and customizations on **any Arch-based distribution**.

> **Fork Info:** I previously collaborated with DarkXero and have always appreciated the quality of the XeroLinux project. I wanted a version that was minimal enough to use as my daily system, but after discussing the idea with him, it became clear that he did not plan to create a minimal edition or make the toolkit available outside the official distribution. Because of that, I decided to take on the work myself and bring XeroLinux features to a minimal Arch installation.
>
> This fork fulfills that goal by providing an installation process that removes the distribution check and introduces additional features

---

## 🎯 What It Does

This tool lets you easily manage and customize your Arch-based system through a clean, modern interface:

* **Update your system** with a single click
* **Install package managers** - Octopi, Bauh, Warehouse, Flatseal, and more
* **Set up drivers** - GPU drivers (NVIDIA, AMD), Tailscale VPN, ASUS ROG tools
* **Configure gaming** - Steam with dependencies, Lutris, Heroic, Bottles, Gamescope, Falcond
* **Customize your desktop** - ZSH setup, GRUB themes, Plymouth, desktop themes
* **Manage containers & VMs** - Docker, Podman, VirtualBox, DistroBox, KVM/QEMU
* **Install multimedia tools** - OBS Studio, Jellyfin, and more
* **Service your system** - Clear caches, fix keyrings, update mirrors, add third-party repos
* **Biometric authentication** - Fingerprint and facial recognition (jailbroken, see Changes below)

---

## 💻 Supported Distributions

Any **Arch-based** distribution:
- Arch Linux
- EndeavourOS
- Manjaro
- CachyOS
- Garuda Linux
- ArcoLinux
- And others...

## ⚙️ Requirements

- **AUR Helper** - Paru or Yay (required for most features)
- **Flatpak** - optional but recommended

## 📦 Installation

**One-liner:**
```sh
rm -rf /tmp/xero-toolkit-open && git clone https://github.com/MurderFromMars/CyberXero-Toolkit.git /tmp/xero-toolkit-open && sh /tmp/xero-toolkit-open/install.sh && rm -rf /tmp/xero-toolkit-open
```

**Manual:**
```bash
git clone https://github.com/MurderFromMars/CyberXero-Toolkit.git
cd CyberXero-Toolkit
./install.sh
```

The installer will:
1. Install build dependencies via pacman
2. Build from source using Cargo
3. Install to `/opt/xero-toolkit`
4. Create desktop entry and icon

## 🗑️ Uninstallation

```bash
cd CyberXero-Toolkit
./uninstall.sh
```

Or manually:
```bash
sudo rm -rf /opt/xero-toolkit
sudo rm -f /usr/bin/xero-toolkit
sudo rm -f /usr/share/applications/xero-toolkit.desktop
sudo rm -f /usr/share/icons/hicolor/scalable/apps/xero-toolkit.png
```

## 🔧 Build Dependencies

Installed automatically by the installer:
- `rust` & `cargo`
- `pkgconf`
- `gtk4`
- `glib2`
- `libadwaita`
- `vte4`
- `flatpak`
- `polkit`

---

## ✨ Changes from Original

### Distribution Freedom
- **Removed XeroLinux distribution check** - works on any Arch-based distro
- Added `install.sh` for easy building from source
- Added `uninstall.sh` for clean removal

### 🔓 Biometrics — Jailbroken Edition
CyberXero Toolkit has properly integrated biometrics functions *before* upstream!

**Fingerprint Authentication (XFPrintD GUI)**
- Builds from source using a [jailbroken fork](https://github.com/MurderFromMars/xfprintd-gui) that bypasses upstream lockdowns
- Removed distribution checks that blocked installation on non-XeroLinux systems
- Full functionality — enroll fingerprints, manage PAM integration, works with any fprintd-compatible reader

**Facial Recognition (Howdy Qt)**
- First fully working integration  was able to get the jump on upstream due to them packaging it while we build from source
- Fixed broken dependencies — upstream pointed to `howdy-bin` which fails to build; we use `howdy-git` instead
- Builds [xero-howdy-qt](https://github.com/XeroLinuxDev/xero-howdy-qt) from source with correct dependencies

**Install AND uninstall buttons** for both tools — proper lifecycle management, another CyberXero-first feature

### Smart Mirror Updates
- **Auto-detects all installed repositories** and updates their mirrorlists automatically
- Supports: Arch, CachyOS, Chaotic-AUR, EndeavourOS, Manjaro, RebornOS, Artix
- Uses `rate-mirrors` for optimal mirror selection
- No manual selection needed — just click and all detected mirrorlists are updated

### Third-Party Repository Installation
Added buttons in the **Servicing / System Tweaks** page to easily add popular Arch repositories:

- **Install CachyOS Repos** - Adds the [CachyOS](https://cachyos.org/) repositories for performance-optimized packages and kernels
- **Install Chaotic-AUR** - Adds the [Chaotic-AUR](https://aur.chaotic.cx/) repository for pre-built AUR packages
- **Add XeroLinux Repo** - Access to XeroLinux packages without running XeroLinux

### Smart Package Installation
- **Falcond Gaming Utility** - Intelligently checks if packages are available in your configured repos before falling back to AUR
- Automatically uses pacman for repo packages, AUR helper only when needed

### Rebranding
- **Updated About Dialog** - Reflects the fork's origin and enhancements
- **Modified Links** - Discord and YouTube links updated (configurable in `gui/src/config.rs`)
- **Logo** - Changed to a more appropriate Arch logo

---

## 📄 License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## 🙏 Credits

- Original [XeroLinux Toolkit](https://github.com/xerolinux/xero-toolkit) by the XeroLinux team
- [XeroLinux](https://xerolinux.xyz/) project
- [CachyOS](https://cachyos.org/) for their optimized repositories
- [Chaotic-AUR](https://aur.chaotic.cx/) for pre-built AUR packages

