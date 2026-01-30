# 🛠️ Xero Toolkit Open

A GTK4 GUI application for managing system tools, configurations, and customizations on **any Arch-based distribution**.

> **Fork Info:** This is a "jailbroken" version of [XeroLinux Toolkit](https://github.com/synsejse/xero-toolkit) that removes the XeroLinux-only restriction, allowing it to work on Arch, EndeavourOS, Manjaro, CachyOS, and other Arch-based systems.

## 📸 Screenshots

![Main Page](screenshots/main_page.png)
*Main application window*

![Installation Dialog](screenshots/installing_dialog.png)
*Real-time progress tracking during package installation*

![Selection Dialog](screenshots/selection_dialog.png)
*Multi-select interface for choosing tools and applications to install*

## 🎯 What It Does

This tool lets you easily manage and customize your Arch-based system through a clean, modern interface:

* **Update your system** with a single click
* **Install package managers** - Octopi, Bauh, Warehouse, Flatseal, and more
* **Set up drivers** - GPU drivers (NVIDIA, AMD), Tailscale VPN, ASUS ROG tools
* **Configure gaming** - Steam with dependencies, Lutris, Heroic, Bottles, Gamescope
* **Customize your desktop** - ZSH setup, GRUB themes, Plymouth, desktop themes
* **Manage containers & VMs** - Docker, Podman, VirtualBox, DistroBox, KVM/QEMU
* **Install multimedia tools** - OBS Studio, Jellyfin, and more
* **Service your system** - Clear caches, fix keyrings, update mirrors

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
rm -rf /tmp/xero-toolkit-open && git clone https://github.com/MurderFromMars/xero-toolkit-open.git /tmp/xero-toolkit-open && sh /tmp/xero-toolkit-open/install.sh && rm -rf /tmp/xero-toolkit-open
```

**Manual:**
```bash
git clone https://github.com/MurderFromMars/xero-toolkit-open.git
cd xero-toolkit-open
./install.sh
```

The installer will:
1. Install build dependencies via pacman
2. Patch the XeroLinux distribution check
3. Build from source using Cargo
4. Install to `/opt/xero-toolkit`
5. Create desktop entry and icon

## 🗑️ Uninstallation

```bash
cd xero-toolkit-open
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
- `scx-tool` (AUR)

## ✨ Changes from Original

- Removed XeroLinux distribution check
- Added install.sh for easy building on any Arch-based distro
- Added uninstall.sh

## 📄 License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## 🙏 Credits

- Original [XeroLinux Toolkit](https://github.com/synsejse/xero-toolkit) by [synsejse](https://github.com/synsejse)
- [XeroLinux](https://xerolinux.xyz/) team
