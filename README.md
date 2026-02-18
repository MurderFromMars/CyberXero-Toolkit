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
- **XeroLinux Repo FOR A COUPLE FINCTIONS*** - a couple functions need access to metapackages on the xerolinux repo. (like some the VM stuff,) I'm looking into resolving this but for now the forked toolkit does have ability to add the xerolinux repo to your system 

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
✨ Changes from Original
Distribution Freedom

Removed XeroLinux distribution check at the source level — the check is fully deleted from the codebase, not patched at runtime. Works cleanly on any Arch-based distro with no hacks.
The old install.sh patched a sed expression over the binary check at build time. This was fragile and broke when the upstream code restructured. The check is now simply gone from the source.
Added install.sh for easy building from source
Added uninstall.sh for clean removal

🔧 Build & Dependency Fixes

Migrated from deprecated glib::MainContext::channel API to async_channel — the old synchronous GLib channel API was removed in glib-rs 0.19. System dependency checks now run on a background thread and report back via an async channel, keeping the UI responsive during startup.
Added async-channel = "2" dependency to gui/Cargo.toml
Removed dead code — cleaned up unused re-exports and constants (check_system_requirements, XEROLINUX_CHECK) that were left over after the distribution check removal, eliminating all compiler warnings
Window presentation fix — the main window now only presents after the full UI is assembled, preventing a visible resize/flash on tiling window managers

🔓 Biometrics — Jailbroken Edition
Bringing the latest upstream toolkit updates, with none of the restrictions!
Fingerprint Authentication (XFPrintD GUI)

Builds from source using a jailbroken fork that bypasses upstream lockdowns
Removed distribution checks that blocked installation on non-XeroLinux systems
Full functionality — enroll fingerprints, manage PAM integration, works with any fprintd-compatible reader

Facial Recognition (Howdy Qt)

First fully working integration — able to get the jump on upstream due to them packaging it while we build from source
Fixed broken dependencies — upstream pointed to howdy-bin which fails to build; we use howdy-git instead
Builds xero-howdy-qt from source with correct dependencies

Install AND uninstall buttons for easy removal, another added feature unique to this fork
xPackageManager Integration
Added a forked version of the new xPackageManager that has had the distro check removed and repo hard coding replaced with a dynamic system that allows it to work with any repos on the system.
Smart Mirror Updates

Auto-detects all installed repositories and updates their mirrorlists automatically
Supports: Arch, CachyOS, Chaotic-AUR, EndeavourOS, Manjaro, RebornOS, Artix
Uses rate-mirrors for optimal mirror selection
No manual selection needed — just click and all detected mirrorlists are updated

Third-Party Repository Installation
Added buttons in the Servicing / System Tweaks page to easily add popular Arch repositories:

Install CachyOS Repos - Adds the CachyOS repositories for performance-optimized packages and kernels
Install Chaotic-AUR - Adds the Chaotic-AUR repository for pre-built AUR packages
Add XeroLinux Repo - Access to XeroLinux packages without running XeroLinux

Smart Package Installation

Falcond Gaming Utility - Intelligently checks if packages are available in your configured repos before falling back to AUR
Automatically uses pacman for repo packages, AUR helper only when needed. Also added the new falcond-gui app.

Containers & VMs — Fully Rewritten
The entire Containers & VMs page has been overhauled to remove dependency on XeroLinux meta-packages and add proper install/uninstall support for every tool.
No more meta-packages — the original used virtualbox-meta and virt-manager-meta which are XeroLinux-specific and unavailable on other distros. Every tool now installs an explicit, documented package list that works on any Arch-based system.
Uninstall buttons added for every tool — Docker, Podman, VirtualBox, DistroBox, KVM/QEMU, and the iOS iPA Sideloader all have a dedicated uninstall button that properly cleans up services, groups, and packages.
Smart state tracking — install buttons grey out with a ✓ when a tool is already installed and refresh automatically when you return to the window, so the UI always reflects reality.
VirtualBox — kernel-aware host modules
The old code just ran virtualbox-meta. The new code reads uname -r at install time and selects the right host module:

-arch kernel → virtualbox-host-modules-arch (prebuilt)
-lts kernel → virtualbox-host-modules-lts (prebuilt)
Any custom kernel (zen, cachyos, hardened, etc.) → virtualbox-host-dkms + matching kernel headers auto-detected from the version string

Uninstall dynamically detects whichever host module variant is present and removes it cleanly.
KVM/QEMU — explicit packages, conflict resolution, CPU-aware nested virt
Replaces virt-manager-meta with a full explicit package list: qemu-desktop, libvirt, virt-manager, virt-viewer, edk2-ovmf, dnsmasq, iptables-nft, openbsd-netcat, swtpm.
Notable additions:

Detects and resolves iptables / gnu-netcat conflicts before installing (these clash with iptables-nft and openbsd-netcat)
Reads /proc/cpuinfo to write the correct nested virtualisation config (kvm-intel vs kvm-amd)
Adds the user to the libvirt group automatically
Enables and starts libvirtd.service as part of the install sequence
swtpm included for Windows 11 VM compatibility (TPM 2.0)

Uninstall stops and disables all libvirtd services/sockets, removes the user from the libvirt group, and cleans up nested virt modprobe configs.
Docker — uninstall properly stops services and removes user from docker group before removing packages.
Podman — uninstall stops the podman socket, removes Podman Desktop flatpak if present, then removes packages.
DistroBox — uninstall removes BoxBuddy flatpak if present before removing the package.
Bundles XeroLinux Extra-Scripts Package
Various scripts needed for things like the updater to work are included in this repo and installed automatically alongside the toolkit.
Rebranding

Updated About Dialog - Reflects the fork's origin and enhancements
Modified Links - Discord and YouTube links updated (configurable in gui/src/config.rs)
Logo - Changed to a more appropriate Arch logo

---

## 📄 License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## 🙏 Credits

- Original [XeroLinux Toolkit](https://github.com/synsejse/xero-toolkit) by Synse and DarkZero
- [XeroLinux](https://xerolinux.xyz/) project
- [CachyOS](https://cachyos.org/) for their optimized repositories
- [Chaotic-AUR](https://aur.chaotic.cx/) for pre-built AUR packages

