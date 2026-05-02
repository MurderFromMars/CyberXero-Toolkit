# CyberXero Toolkit

A GTK4/libadwaita system management toolkit for Arch-based Linux distributions. Built in Rust. Handles everything from driver installation and container setup to emulator configuration and desktop theming — all from one application.

> CyberXero runs on any distribution with `pacman`. Features are driven by explicit package lists rather than distribution-specific metapackages, so the same build works across the Arch ecosystem. SystemD is required, so no Artix support currently.

---

## Pages

### Containers & VMs
Full virtualization and container management.

- **VirtualBox** — detects your running kernel and installs the correct host modules (prebuilt for `linux`/`linux-lts`, DKMS with auto-detected headers for everything else)
- **KVM/QEMU** — complete package stack with conflict detection (`iptables`/`gnu-netcat`), CPU vendor detection for nested virtualization, `libvirt` group membership, service enablement, and `swtpm` for Windows 11 TPM 2.0
- **Docker & Podman** — install with service enablement and group setup
- **DistroBox** — containerized Linux environments
- **iOS iPA Sideloader** — sideload apps to iOS devices
- Every tool has dedicated install and uninstall buttons with smart state tracking

### Emulators
One-click install for:

- **RetroArch** — with 29 selectable libretro cores spanning NES, SNES, GB/GBC, GBA, N64, DS, PS1, Genesis, Saturn, Dreamcast, PC Engine, PSP, Arcade, ScummVM, Atari (2600/5200/7800/Lynx/Jaguar), 3DO, Amiga, C64, DOS, Neo Geo Pocket, and WonderSwan
- **Standalone emulators** — DuckStation (PS1), PCSX2 (PS2), RPCS3 (PS3), ShadPS4 (PS4), PPSSPP (PSP), Vita3K (PS Vita), mGBA (GBA), melonDS (DS), Dolphin (GC/Wii), Cemu (Wii U), Ryujinx (Switch), xemu (Xbox), Flycast (Dreamcast), MAME (Arcade)

### Servicing & System Tweaks
15 system maintenance and configuration tools:

- **Third-party repos** — one-click install for CachyOS, Chaotic-AUR, and XeroLinux repositories
- **Smart mirror updates** — auto-detects all configured repos and updates mirrorlists via `rate-mirrors` (Arch, CachyOS, Chaotic-AUR, EndeavourOS, Manjaro, RebornOS, Artix)
- **xPackageManager** — dynamic repo detection
- **Toolkit self-updater** — checks upstream commit hash and rebuilds from source
- **Orphaned package removal** — presents a selectable list of orphans to clean up
- System updater, keyring repair, pacman.conf management, and more

### Drivers
GPU driver management with guided installation:

- **NVIDIA** — driver installation with automatic `mkinitcpio` and GRUB configuration
- **GPU driver management** for AMD, Intel, and NVIDIA

### Multimedia
- **GPU Screen Recorder** — smart repo detection, installs from official repos when available, falls back to AUR
- **OBS Studio** and related multimedia tools
- **Streaming Services** — creates Chrome web app shortcuts for streaming platforms (Netflix, Hulu, Disney+, etc.) with automatic Steam integration on handheld devices. Rust reimplementation of [HandheldStreamingServiceUtility](https://github.com/MurderFromMars/HandheldStreamingServiceUtility)
- **Enhanced Audio** — PipeWire spatial audio convolver with selectable intensity levels, multi-sink support, and a suspend/resume audio fix service. Rust reimplementation of [Enhanced-Handheld-Audio](https://github.com/MurderFromMars/Enhanced-Handheld-Audio)

Both started as handheld-focused projects and have been extended here to work on desktops too.

### Biometrics
- **Fingerprint authentication** — full PAM integration, works with any `fprintd`-compatible reader
- **Facial recognition (Howdy Qt)** — built from `howdy-git` with correct dependencies
- Both ship with install and uninstall support

### Gaming Tools
Gaming-related utilities and optimizations for Linux gaming.

### Gamescope
Gamescope session and compositor configuration.

### Customization
Desktop theming and customization tools, including the bundled `cyberxero-theme` installer with backup/restore.

---

## Bundled Scripts

13 system scripts bundled directly in the repo so every feature works regardless of distribution:

| Script | Purpose |
|---|---|
| `upd` | System updater: pacman, AUR, Flatpak, Rust toolchain, firmware. Detects if reboot is needed |
| `xpm` | Plymouth theme wizard |
| `cyberxero-theme` | Desktop theme installer with backup/restore |
| `PS4-theme` | PS4-style desktop theme |
| `Lunar-Glass` | Lunar Glass desktop theme |
| `rddav` | Real-Debrid WebDAV automount via rclone + systemd |
| `gcm` | Git credential helper wizard |
| `pmpd` | Pamac database repair |
| `pacup` | Pacman.conf updater with automatic backup |
| `keyfix` | Pacman keyring and database repair |
| `rpipe` | PipeWire restart utility |
| `opr-drv` | OpenRazer driver installer with user group setup |
| `getcider` | Cider music player installer with GPG key signing |

All scripts install to `/usr/local/bin` and are removed cleanly by the uninstaller.

---

## Supported Distributions

Any **Arch-based** distribution:

Arch Linux - EndeavourOS - CachyOS - Garuda Linux - Manjaro - ArcoLinux - RebornOS

If it has `pacman`, it runs. (Unless you're using Artix, in which case you have bigger fish to fry KIDDING i might add support if i get enough requests for it it's a lot of work though)

CyberXero Toolkit ships as the default system management tool in [OrbitOS](https://github.com/MurderFromMars/OrbitOS), my custom Arch Linux spin.

## Requirements

- **AUR helper** — Paru or Yay (the installer will offer to set one up if missing)
- **Flatpak** — optional, used for some multimedia tools

## Installation

**One-liner:**
```sh
rm -rf /tmp/cyberxero-toolkit-open && git clone https://github.com/MurderFromMars/CyberXero-Toolkit.git /tmp/cyberxero-toolkit-open && sh /tmp/cyberxero-toolkit-open/install.sh && rm -rf /tmp/cyberxero-toolkit-open
```

**Manual:**
```bash
git clone https://github.com/MurderFromMars/CyberXero-Toolkit.git
cd CyberXero-Toolkit
./install.sh
```

The installer handles dependency resolution, AUR helper setup, Rust compilation, binary installation to `/opt/cyberxero-toolkit`, desktop entry creation, icon registration, and script deployment.

## Uninstallation

```bash
cd CyberXero-Toolkit
./uninstall.sh
```

Removes binaries, symlinks, desktop entries, icons, all bundled scripts, and autostart entries.

## Build Dependencies

Installed automatically by `install.sh`:

`rust` - `cargo` - `pkgconf` - `gtk4` - `glib2` - `libadwaita` - `vte4` - `flatpak` - `polkit` - `base-devel` - `scx-scheds` - `scx-tools`

---

## License

GNU General Public License v3.0 — see [LICENSE](LICENSE).

---

## Credit for Original Concept

This project started off as a fork of the Xero Toolkit, a tool made by Synse and DarkXero, it started off as seeking to port the toolkit so it could be used on other distros, lots of drama ensued, over time, this project has become distinct enough (and given how much the relationship between me and upstream has soured) remaining a fork no longer makes sense, I've rewritten entire pages, added new ones. and made lots of other additions and rewritten a significant amount of the backend code. so this is now a standalone project.
