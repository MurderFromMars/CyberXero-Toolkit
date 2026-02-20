# CyberXero Toolkit

**The Xero Toolkit, unchained.** A GTK4/libadwaita system management application for *any* Arch-based distribution, not just one.

Built in Rust. Ships with 11 system utility scripts. Adds an entire new feature page. Rewrites two others from the ground up. Strips out every distribution lock. And it does all of this while fixing the upstream codebase's ineffecient architecture.

> **Origin:** This is a hard fork of the [XeroLinux Toolkit](https://github.com/synsejse/xero-toolkit) by Synse and DarkZero. The original is a competent tool, if you happen to be running XeroLinux. If you aren't, it refuses to start. CyberXero removes that restriction at the source level, then goes dramatically further: rewriting entire subsystems, adding several features the original never shipped, and delivering a toolkit that treats every Arch-based system as a first class citizen.

---

## What Changed — And How Much

This is not a cosmetic fork. Here is what the numbers look like:

| Metric | Original Toolkit | CyberXero Toolkit |
|---|---|---|
| Total codebase (Rust + Bash + UI) | 13,000 lines | **15,200 lines** |
| Rust source | 9,749 lines | **11,258 lines** |
| Containers & VMs page | 290 lines | **839 lines** (complete rewrite) |
| Servicing & System Tweaks page | 250 lines | **1,261 lines** (5× expansion) |
| Biometrics page | 136 lines | **325 lines** (jailbroken + uninstall) |
| Bundled system scripts | 0 | **11 scripts, 1,828 lines** |
| Supported distributions | 1 (XeroLinux) | **All Arch-based** |
| Uninstall support for installed tools | Partial | **Every single tool** |
| Install/uninstall scripts for the toolkit itself | None | **Both included** |

The original toolkit will not run on your system unless it reads "XeroLinux" from `/etc/os-release`. the check is **deleted from the source**. There is no runtime gate. There is no workaround. The code simply isn't there.

---

## Feature Breakdown.

### Containers & VMs, Fully Rewritten
The original Containers & VMs page relies on `virtualbox-meta` and `virt-manager-meta`, which are XeroLinux-specific metapackages that **do not exist** on any other distribution. CyberXero replaces every single one with explicit, documented package lists that work everywhere.

What this means in practice:

- **VirtualBox** now detects your running kernel at install time. Stock `linux` gets prebuilt host modules. `linux-lts` gets its own. Any custom kernel (zen, cachyos, hardened, etc.) gets DKMS with auto-detected headers. The original just ran a metapackage and hoped for the best.
- **KVM/QEMU** installs a complete, explicit package list: `qemu-desktop`, `libvirt`, `virt-manager`, `virt-viewer`, `edk2-ovmf`, `dnsmasq`, `iptables-nft`, `openbsd-netcat`, `swtpm`. It detects and resolves `iptables`/`gnu-netcat` conflicts before they break your install. It reads `/proc/cpuinfo` to write the correct nested virtualization config (`kvm-intel` vs `kvm-amd`). It adds you to the `libvirt` group. It enables `libvirtd.service`. It ships `swtpm` for Windows 11 TPM 2.0 compatibility. The original did none of this.
- **Every tool** (Docker, Podman, VirtualBox, DistroBox, KVM/QEMU, iOS iPA Sideloader) now has a dedicated **uninstall button** that properly stops services, removes group memberships, and cleans up packages.
- **Smart state tracking**: install buttons grey out with a checkmark when a tool is already detected, and refresh automatically when you return to the page.

### Servicing & System Tweaks  5× Expansion
The original servicing page has 7 functions. CyberXero has **15**, including:

- **Third-party repo installation**: one-click buttons for CachyOS repos, Chaotic-AUR, and the XeroLinux repo (so you can access XeroLinux packages without running XeroLinux).
- **Smart mirror updates**: auto-detects every installed repository and updates all mirrorlists using `rate-mirrors`. Supports Arch, CachyOS, Chaotic-AUR, EndeavourOS, Manjaro, RebornOS, and Artix out of the box.
- **xPackageManager integration**: a forked version with the distro check removed and hardcoded repo references replaced with dynamic detection, works with whatever repos you actually have configured.
- **Toolkit self update**: checks the upstream commit hash and rebuilds from source when a new version is available.

### Multimedia Page Addition
- **GPU Screen Recorder** with smart repo detection  installs from official repos when available, falls back to AUR when not.

### Biometrics — Jailbroken Edition
The original ships biometric support that is locked to XeroLinux. CyberXero jailbreaks it:

- **Fingerprint authentication** builds from a fork that bypasses upstream lockdowns. Full PAM integration. Works with any `fprintd`-compatible reader.
- **Facial recognition (Howdy Qt)** is the first fully working integration — CyberXero beat upstream to it by building from source rather than depending on a broken `howdy-bin` package. Uses `howdy-git` with correct dependencies.
- Both features ship with **install and uninstall buttons** — the original has no uninstall path.

### 11 Bundled System Scripts
The original toolkit calls out to scripts that are packaged separately on XeroLinux  scripts that simply don't exist on any other distribution. 10 of these 11 are those XeroLinux packaged utilities, bundled directly into this repo so that every feature they power actually works regardless of what distro you're on. The 11th, `cyberxero-theme`, is entirely new:

| Script | Purpose |
|---|---|
| `upd` | Comprehensive system updater: pacman, AUR, Flatpak, Rust toolchain, firmware. Detects if reboot is needed. |
| `xpm` | Plymouth theme wizard for Arch Linux |
| `cyberxero-theme` | CyberXero desktop theme installer with backup/restore (765 lines) |
| `rddav` | Real-Debrid WebDAV automount via rclone + systemd |
| `gcm` | Git credential helper wizard |
| `pmpd` | Pamac database repair tool |
| `pacup` | Pacman.conf updater with automatic backup |
| `keyfix` | Pacman keyring and database repair |
| `rpipe` | PipeWire restart utility |
| `opr-drv` | OpenRazer driver installer with user group setup |
| `getcider` | Cider music player installer with GPG key signing |

All scripts are installed to `/usr/local/bin` automatically and removed cleanly by the uninstaller.

### Under the Hood

- **Async channel migration**: the original uses the deprecated `glib::MainContext::channel` API, which was removed in `glib-rs` 0.19. CyberXero migrates to `async_channel`, putting system dependency checks on a background thread. The result is a responsive UI during startup, the original hitches noticeably, and it got worse the more features I added, so this rewrite was much needed.
- **Window presentation fix**: the main window now only presents after the full UI is assembled, preventing the visible resize flash that plagued tiling window manager/krohnkite users.
- **Smart package detection**: Falcond and other tools check if packages exist in your configured repos before falling back to AUR, using `pacman` for repo packages and your AUR helper only when necessary.

---

## Supported Distributions

Any **Arch-based** distribution, including but not limited to:

Arch Linux · EndeavourOS · CachyOS · Garuda Linux · Manjaro · ArcoLinux · Artix · RebornOS

If it has `pacman`, it runs.

## Requirements

- **AUR helper** — Paru or Yay (the installer will offer to set one up if missing)
- **Flatpak** — optional but recommended for OBS Studio and some multimedia tools

## Installation

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

The installer handles everything: dependency resolution, AUR helper setup, Rust compilation, binary installation to `/opt/xero-toolkit`, desktop entry creation, icon registration, and deployment of all 11 system scripts.

## Uninstallation

```bash
cd CyberXero-Toolkit
./uninstall.sh
```

Removes binaries, symlinks, desktop entries, icons, all bundled scripts, and user autostart entries. Clean removal — nothing left behind.

## Build Dependencies

Installed automatically by `install.sh`:

`rust` · `cargo` · `pkgconf` · `gtk4` · `glib2` · `libadwaita` · `vte4` · `flatpak` · `polkit` · `base-devel` · `scx-scheds`

---

## Screenshots

![Main Page](screenshots/main_page.png)
*Main application window*

![Installation Dialog](screenshots/installing_dialog.png)
*Real-time terminal output during package operations*

![Selection Dialog](screenshots/selection_dialog.png)
*Multi-select interface for tool installation*

---

## License

GNU General Public License v3.0 — see [LICENSE](LICENSE).

## Credits

- Original [XeroLinux Toolkit](https://github.com/synsejse/xero-toolkit) by Synse and DarkZero
- [XeroLinux](https://xerolinux.xyz/) project
- [CachyOS](https://cachyos.org/) for their optimized repositories
- [Chaotic-AUR](https://aur.chaotic.cx/) for pre-built AUR packages
