# 🛠️ Xero Toolkit Open

A GTK4 GUI application for managing system tools, configurations, and customizations on **any Arch-based distribution**.

> **Fork Info:** This is a "jailbroken" version of [XeroLinux Toolkit](https://github.com/synsejse/xero-toolkit) that removes the XeroLinux-only restriction, allowing it to work on Arch, EndeavourOS, Manjaro, CachyOS, and other Arch-based systems. adds a couple new improvements, and has been slightly rebranded to differentiate itself from the original


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
- `scx-scheds`

## ✨ Changes from Original

### Distribution Freedom
- Removed XeroLinux distribution check - works on any Arch-based distro
- Added `install.sh` for easy building from source
- Added `uninstall.sh` for clean removal

### New Features: Third-Party Repository Installation
Added buttons in the **Servicing / System Tweaks** page to easily add popular Arch repositories:

- **Install CachyOS Repos** - Adds the [CachyOS](https://cachyos.org/) repositories, providing access to performance-optimized packages, kernels, and tools like Falcond
- **Install Chaotic-AUR** - Adds the [Chaotic-AUR](https://aur.chaotic.cx/) repository, providing pre-built AUR packages for faster installation

### Smart Package Installation
- **Falcond Gaming Utility** - Now intelligently checks if packages are available in your configured repos (e.g., CachyOS) before falling back to AUR
  - Installs `falcond`, `falcond-gui`, `falcond-profiles`, and `tuned-ppd`
  - Automatically uses pacman for repo packages, AUR helper only when needed

### Rebranding
The fork has been lightly rebranded to reflect its enhanced/jailbroken status:

- **Updated About Dialog** - Reflects the fork's origin and enhancements
- **Modified Links** - Discord and YouTube links updated (configurable in `gui/src/config.rs`)
- **Logo** - Can be replaced at `gui/resources/icons/apps/xero-toolkit.png` (and create `gui/resources/icons/scalable/apps/` directory for the build)

To customize links, edit `gui/src/config.rs`:
```rust
pub mod links {
    pub const DISCORD: &str = "https://your-discord-link/";
    pub const YOUTUBE: &str = "https://your-youtube-link/";
    pub const WEBSITE: &str = "https://your-website/";
    pub const DONATE: &str = "https://your-donate-link/";
}
```

## 📄 License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## 🙏 Credits

- Original [XeroLinux Toolkit](https://github.com/synsejse/xero-toolkit) by [synsejse](https://github.com/synsejse)
- [XeroLinux](https://xerolinux.xyz/) team
- [CachyOS](https://cachyos.org/) for their optimized repositories
- [Chaotic-AUR](https://aur.chaotic.cx/) for pre-built AUR packages
