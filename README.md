# 🛠️ XeroLinux Toolkit

A comprehensive GTK4 GUI application for managing system tools, configurations, and customizations on XeroLinux systems.

## 📸 Screenshots

![Main Page](screenshots/main_page.png)
*Main application window*

![Installation Dialog](screenshots/installing_dialog.png)
*Real-time progress tracking during package installation*

![Selection Dialog](screenshots/selection_dialog.png)
*Multi-select interface for choosing tools and applications to install*

## 🎯 What It Does

This tool lets you easily manage and customize your XeroLinux system through a clean, modern interface. You can:

- **Update your system** with a single click
- **Install package managers** - Octopi, Bauh, Warehouse, Flatseal, and more
- **Set up drivers** - GPU drivers (NVIDIA, AMD), Tailscale VPN, ASUS ROG tools
- **Configure gaming** - Steam with dependencies, Lutris, Heroic, Bottles, Gamescope
- **Customize your desktop** - ZSH setup, GRUB themes, Plymouth, desktop themes
- **Manage containers & VMs** - Docker, Podman, VirtualBox, DistroBox, KVM/QEMU
- **Install multimedia tools** - OBS Studio, Jellyfin, and more
- **Service your system** - Clear caches, fix keyrings, update mirrors

## ⚙️ How It Works

The application is split into two parts:

- **GUI Application**: The main interface you interact with for managing your system
- **Authentication Daemon**: Handles privileged operations that require admin rights

When you install packages or run system operations, you'll see live updates showing your progress:
- Real-time command output with colored terminal feedback
- Progress indicators for installations
- And helpful error messages if something needs attention

## ✨ Features

- **Tabbed navigation** with organized categories
- **Smart dependency detection** - shows which packages are already installed
- **Multi-select installations** - install related tools together
- **AUR helper support** - works with Paru or Yay
- **Flatpak integration** - manage both native and Flatpak packages
- **Modern GTK4 interface** that fits naturally in your desktop

## 💻 System Requirements

- **XeroLinux** — primary supported platform. The tool may run on other distributions, but those will receive a limited‑support notice at startup; support for non‑XeroLinux systems is best‑effort and not guaranteed.
- **AUR Helper** - Paru or Yay (required for most features)
- **Flatpak** - optional but recommended

This tool is designed primarily for XeroLinux. It may run on other distributions, but you will receive a limited-support notice at startup and some features may not behave as expected. Because some features depend on distribution-specific components, the app enforces critical dependency checks at startup and will prompt you to resolve any missing requirements before you can continue.

## Forks, affiliation & support

This repository is the original source of the project. Community forks and rebranded copies exist independently and are not affiliated with this repository or its maintainers — if you're using one, please direct support requests to the respective fork's maintainers.

### Why distribution checks exist
The startup dependency checks and limited-support notice are there to set clear expectations: this tool is built and maintained primarily for XeroLinux, and some features rely on distribution-specific components. These checks help avoid hard-to-diagnose issues that fall outside the scope of the project.

### What's supported
Official support, bug fixes, and releases target XeroLinux. Running the tool on other distributions may work, and we'll help where practical, but this is a hobby project — support for non-XeroLinux systems is best-effort and not guaranteed.

## 📄 License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE](LICENSE) file for details.
