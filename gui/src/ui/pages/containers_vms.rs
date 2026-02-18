//! Containers and VMs page button handlers.
//!
//! All package lists are fully explicit — no XeroLinux meta-packages
//! (virtualbox-meta, virt-manager-meta) are used, ensuring compatibility
//! with any Arch-based distribution.
//!
//! Handles install + uninstall for:
//! - Docker
//! - Podman (with optional Podman Desktop flatpak)
//! - VirtualBox (kernel-aware host modules / dkms)
//! - DistroBox (with BoxBuddy flatpak)
//! - KVM / QEMU / virt-manager (with conflict resolution & nested virt)
//! - iOS iPA Sideloader (Plume Impactor flatpak)

use crate::core;
use crate::ui::dialogs::selection::{
    show_selection_dialog, SelectionDialogConfig, SelectionOption, SelectionType,
};
use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::extract_widget;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button};
use log::info;

// ─── Shared helpers ─────────────────────────────────────────────────────────

/// Update install / uninstall button pair based on installation status.
///
/// Installed  → install button greyed with "✓", uninstall visible.
/// Not installed → install button active, uninstall hidden.
fn update_button_state(
    install_button: &Button,
    uninstall_button: &Button,
    is_installed: bool,
    default_label: &str,
) {
    if is_installed {
        install_button.set_label(&format!("{} ✓", default_label));
        install_button.set_sensitive(false);
        install_button.remove_css_class("suggested-action");
        install_button.add_css_class("dim-label");
        uninstall_button.set_visible(true);
    } else {
        install_button.set_label(default_label);
        install_button.set_sensitive(true);
        install_button.add_css_class("suggested-action");
        install_button.remove_css_class("dim-label");
        uninstall_button.set_visible(false);
    }
}

/// Build a `-Rns` argument list that only includes packages actually installed.
/// Prevents pacman from erroring on packages that were already removed or
/// never installed in the first place.
fn removable_packages(candidates: &[&str]) -> Vec<String> {
    candidates
        .iter()
        .filter(|pkg| core::is_package_installed(pkg))
        .map(|pkg| pkg.to_string())
        .collect()
}

/// Check all install states off the main thread, then update all button pairs
/// at once. Called on initial page load and on window refocus — never blocks
/// the GTK main loop.
fn async_refresh_states(
    docker: (Button, Button),
    podman: (Button, Button),
    vbox: (Button, Button),
    distrobox: (Button, Button),
    kvm: (Button, Button),
    ipa: (Button, Button),
) {
    let (tx, rx) = async_channel::bounded::<(bool, bool, bool, bool, bool, bool)>(1);

    std::thread::spawn(move || {
        let _ = tx.send_blocking((
            core::is_package_installed("docker"),
            core::is_package_installed("podman"),
            core::is_package_installed("virtualbox"),
            core::is_package_installed("distrobox"),
            core::is_package_installed("virt-manager"),
            core::is_flatpak_installed("dev.khcrysalis.PlumeImpactor"),
        ));
    });

    gtk4::glib::MainContext::default().spawn_local(async move {
        if let Ok((d, p, v, db, k, ipa_ok)) = rx.recv().await {
            update_button_state(&docker.0, &docker.1, d, "Docker");
            update_button_state(&podman.0, &podman.1, p, "Podman");
            update_button_state(&vbox.0, &vbox.1, v, "Virtual Box");
            update_button_state(&distrobox.0, &distrobox.1, db, "DistroBox");
            update_button_state(&kvm.0, &kvm.1, k, "Qemu Virtual Manager");
            update_button_state(&ipa.0, &ipa.1, ipa_ok, "iOS iPA Sideloader");
        }
    });
}

// ─── Page entry point ───────────────────────────────────────────────────────

/// Set up all button handlers for the containers/VMs page.
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    let docker_btns = setup_docker(page_builder, window);
    let podman_btns = setup_podman(page_builder, window);
    let vbox_btns = setup_vbox(page_builder, window);
    let distrobox_btns = setup_distrobox(page_builder, window);
    let kvm_btns = setup_kvm(page_builder, window);
    let ipa_btns = setup_ipa_sideloader(page_builder, window);

    // Single async pass to set initial button states — no main-thread blocking.
    async_refresh_states(
        docker_btns.clone(),
        podman_btns.clone(),
        vbox_btns.clone(),
        distrobox_btns.clone(),
        kvm_btns.clone(),
        ipa_btns.clone(),
    );

    // Refresh states whenever the user returns focus to the window.
    window.connect_is_active_notify(move |w| {
        if w.is_active() {
            async_refresh_states(
                docker_btns.clone(),
                podman_btns.clone(),
                vbox_btns.clone(),
                distrobox_btns.clone(),
                kvm_btns.clone(),
                ipa_btns.clone(),
            );
        }
    });
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Docker
// ═══════════════════════════════════════════════════════════════════════════════

/// Core packages for a working Docker setup.
const DOCKER_PACKAGES: &[&str] = &["docker", "docker-compose", "docker-buildx"];

fn setup_docker(builder: &Builder, window: &ApplicationWindow) -> (Button, Button) {
    let btn_install = extract_widget::<Button>(builder, "btn_docker");
    let btn_uninstall = extract_widget::<Button>(builder, "btn_docker_uninstall");

    // ── Install ──────────────────────────────────────────────────────────
    let window_clone = window.clone();
    btn_install.connect_clicked(move |_| {
        info!("Docker install button clicked");

        let user = crate::config::env::get().user.clone();

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&[
                        "-S", "--noconfirm", "--needed",
                        "docker", "docker-compose", "docker-buildx",
                    ])
                    .description("Installing Docker engine and tools...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["enable", "--now", "docker.service"])
                    .description("Enabling Docker service...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("groupadd")
                    .args(&["-f", "docker"])
                    .description("Ensuring docker group exists...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("usermod")
                    .args(&["-aG", "docker", &user])
                    .description("Adding your user to docker group...")
                    .build(),
            )
            .build();

        task_runner::run(window_clone.upcast_ref(), commands, "Docker Setup");
    });

    // ── Uninstall ────────────────────────────────────────────────────────
    let window_clone = window.clone();
    btn_uninstall.connect_clicked(move |_| {
        info!("Docker uninstall button clicked");

        let user = crate::config::env::get().user.clone();
        let pkgs = removable_packages(DOCKER_PACKAGES);

        let mut commands = CommandSequence::new()
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["stop", "docker.service", "docker.socket"])
                    .description("Stopping Docker services...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["disable", "docker.service", "docker.socket"])
                    .description("Disabling Docker services...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("gpasswd")
                    .args(&["-d", &user, "docker"])
                    .description("Removing your user from docker group...")
                    .build(),
            );

        if !pkgs.is_empty() {
            let mut args = vec!["-Rns".to_string(), "--noconfirm".to_string()];
            args.extend(pkgs);
            let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            commands = commands.then(
                Command::builder()
                    .aur()
                    .args(&refs)
                    .description("Removing Docker packages and dependencies...")
                    .build(),
            );
        }

        task_runner::run(window_clone.upcast_ref(), commands.build(), "Docker Uninstall");
    });

    (btn_install, btn_uninstall)
}

// ═══════════════════════════════════════════════════════════════════════════════
//  Podman
// ═══════════════════════════════════════════════════════════════════════════════

const PODMAN_PACKAGES: &[&str] = &["podman", "podman-docker"];
const PODMAN_DESKTOP_FLATPAK: &str = "io.podman_desktop.PodmanDesktop";

fn setup_podman(builder: &Builder, window: &ApplicationWindow) -> (Button, Button) {
    let btn_install = extract_widget::<Button>(builder, "btn_podman");
    let btn_uninstall = extract_widget::<Button>(builder, "btn_podman_uninstall");

    // ── Install ──────────────────────────────────────────────────────────
    let window_clone = window.clone();
    btn_install.connect_clicked(move |_| {
        info!("Podman install button clicked");

        let config = SelectionDialogConfig::new(
            "Podman Installation",
            "Podman will be installed. Optionally include the Podman Desktop GUI.",
        )
        .selection_type(SelectionType::Single)
        .selection_required(false)
        .add_option(SelectionOption::new(
            "podman_desktop",
            "Podman Desktop",
            "Graphical interface for managing containers",
            core::is_flatpak_installed(PODMAN_DESKTOP_FLATPAK),
        ))
        .confirm_label("Install");

        let window_for_closure = window_clone.clone();
        show_selection_dialog(window_clone.upcast_ref(), config, move |selected| {
            let mut commands = CommandSequence::new()
                .then(
                    Command::builder()
                        .aur()
                        .args(&["-S", "--noconfirm", "--needed", "podman", "podman-docker"])
                        .description("Installing Podman container engine...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .privileged()
                        .program("systemctl")
                        .args(&["enable", "--now", "podman.socket"])
                        .description("Enabling Podman socket...")
                        .build(),
                );

            if selected.iter().any(|s| s == "podman_desktop") {
                commands = commands.then(
                    Command::builder()
                        .normal()
                        .program("flatpak")
                        .args(&["install", "-y", "flathub", PODMAN_DESKTOP_FLATPAK])
                        .description("Installing Podman Desktop GUI...")
                        .build(),
                );
            }

            if !commands.is_empty() {
                task_runner::run(
                    window_for_closure.upcast_ref(),
                    commands.build(),
                    "Podman Setup",
                );
            }
        });
    });

    // ── Uninstall ────────────────────────────────────────────────────────
    let window_clone = window.clone();
    btn_uninstall.connect_clicked(move |_| {
        info!("Podman uninstall button clicked");

        let mut commands = CommandSequence::new()
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["stop", "podman.socket"])
                    .description("Stopping Podman socket...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["disable", "podman.socket"])
                    .description("Disabling Podman socket...")
                    .build(),
            );

        if core::is_flatpak_installed(PODMAN_DESKTOP_FLATPAK) {
            commands = commands.then(
                Command::builder()
                    .normal()
                    .program("flatpak")
                    .args(&["uninstall", "-y", PODMAN_DESKTOP_FLATPAK])
                    .description("Removing Podman Desktop GUI...")
                    .build(),
            );
        }

        let pkgs = removable_packages(PODMAN_PACKAGES);
        if !pkgs.is_empty() {
            let mut args = vec!["-Rns".to_string(), "--noconfirm".to_string()];
            args.extend(pkgs);
            let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            commands = commands.then(
                Command::builder()
                    .aur()
                    .args(&refs)
                    .description("Removing Podman packages and dependencies...")
                    .build(),
            );
        }

        task_runner::run(
            window_clone.upcast_ref(),
            commands.build(),
            "Podman Uninstall",
        );
    });

    (btn_install, btn_uninstall)
}

// ═══════════════════════════════════════════════════════════════════════════════
//  VirtualBox
// ═══════════════════════════════════════════════════════════════════════════════

/// All possible VirtualBox host modules variants — used during uninstall
/// to clean up whichever one was installed.
const VBOX_HOST_VARIANTS: &[&str] = &[
    "virtualbox-host-modules-arch",
    "virtualbox-host-modules-lts",
    "virtualbox-host-dkms",
];

/// Detect which host modules packages are needed for VirtualBox based on
/// the running kernel (`uname -r`):
///
/// | Kernel suffix | Packages                                             |
/// |---------------|------------------------------------------------------|
/// | `-arch`       | `virtualbox-host-modules-arch` (prebuilt)            |
/// | `-lts`        | `virtualbox-host-modules-lts`  (prebuilt)            |
/// | anything else | `virtualbox-host-dkms` + matching kernel headers     |
///
/// For dkms, the kernel headers package is derived from the version string
/// (e.g. `6.12.8-zen1-1-zen` → `linux-zen-headers`). If the headers
/// package can't be located the install proceeds without it and dkms will
/// prompt the user if needed.
fn detect_vbox_host_packages() -> Vec<String> {
    let uname = std::process::Command::new("uname")
        .arg("-r")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();

    if uname.contains("-arch") {
        vec!["virtualbox-host-modules-arch".to_string()]
    } else if uname.contains("-lts") {
        vec!["virtualbox-host-modules-lts".to_string()]
    } else {
        // Custom kernel (zen, cachyos, hardened, etc.) — needs dkms + headers.
        let mut pkgs = vec!["virtualbox-host-dkms".to_string()];

        if let Some(suffix) = uname.rsplit('-').next() {
            if !suffix.is_empty() && suffix.chars().all(|c| c.is_alphanumeric()) {
                let headers = format!("linux-{}-headers", suffix);
                if core::is_package_in_repos(&headers)
                    || core::is_package_installed(&format!("linux-{}", suffix))
                {
                    pkgs.push(headers);
                }
            }
        }

        pkgs
    }
}

fn setup_vbox(builder: &Builder, window: &ApplicationWindow) -> (Button, Button) {
    let btn_install = extract_widget::<Button>(builder, "btn_vbox");
    let btn_uninstall = extract_widget::<Button>(builder, "btn_vbox_uninstall");

    // ── Install ──────────────────────────────────────────────────────────
    //
    // Packages are listed explicitly instead of using `virtualbox-meta`
    // (XeroLinux-specific) to avoid provider-conflict errors when
    // --noconfirm auto-selects from multiple repos.
    let window_clone = window.clone();
    btn_install.connect_clicked(move |_| {
        info!("VirtualBox install button clicked");

        let host_pkgs = detect_vbox_host_packages();
        info!("Detected VBox host packages: {:?}", host_pkgs);

        let mut install_args: Vec<&str> = vec![
            "-S", "--noconfirm", "--needed",
            "virtualbox",
            "virtualbox-guest-iso",
        ];
        let host_refs: Vec<&str> = host_pkgs.iter().map(|s| s.as_str()).collect();
        install_args.extend_from_slice(&host_refs);

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&install_args)
                    .description("Installing VirtualBox...")
                    .build(),
            )
            .build();

        task_runner::run(window_clone.upcast_ref(), commands, "VirtualBox Setup");
    });

    // ── Uninstall ────────────────────────────────────────────────────────
    //
    // Dynamically checks which host modules variant is present so we
    // clean up regardless of how VBox was originally installed.
    let window_clone = window.clone();
    btn_uninstall.connect_clicked(move |_| {
        info!("VirtualBox uninstall button clicked");

        let mut candidates: Vec<&str> = vec!["virtualbox", "virtualbox-guest-iso"];
        candidates.extend_from_slice(VBOX_HOST_VARIANTS);

        let pkgs = removable_packages(&candidates);
        if pkgs.is_empty() {
            return;
        }

        let mut args = vec!["-Rns".to_string(), "--noconfirm".to_string()];
        args.extend(pkgs);
        let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&refs)
                    .description("Removing VirtualBox and dependencies...")
                    .build(),
            )
            .build();

        task_runner::run(
            window_clone.upcast_ref(),
            commands,
            "VirtualBox Uninstall",
        );
    });

    (btn_install, btn_uninstall)
}

// ═══════════════════════════════════════════════════════════════════════════════
//  DistroBox
// ═══════════════════════════════════════════════════════════════════════════════

const BOXBUDDY_FLATPAK: &str = "io.github.dvlv.boxbuddyrs";

fn setup_distrobox(builder: &Builder, window: &ApplicationWindow) -> (Button, Button) {
    let btn_install = extract_widget::<Button>(builder, "btn_distrobox");
    let btn_uninstall = extract_widget::<Button>(builder, "btn_distrobox_uninstall");

    // ── Install ──────────────────────────────────────────────────────────
    let window_clone = window.clone();
    btn_install.connect_clicked(move |_| {
        info!("DistroBox install button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&["-S", "--noconfirm", "--needed", "distrobox"])
                    .description("Installing DistroBox...")
                    .build(),
            )
            .then(
                Command::builder()
                    .normal()
                    .program("flatpak")
                    .args(&["install", "-y", BOXBUDDY_FLATPAK])
                    .description("Installing BoxBuddy GUI...")
                    .build(),
            )
            .build();

        task_runner::run(window_clone.upcast_ref(), commands, "DistroBox Setup");
    });

    // ── Uninstall ────────────────────────────────────────────────────────
    let window_clone = window.clone();
    btn_uninstall.connect_clicked(move |_| {
        info!("DistroBox uninstall button clicked");

        let mut commands = CommandSequence::new();

        if core::is_flatpak_installed(BOXBUDDY_FLATPAK) {
            commands = commands.then(
                Command::builder()
                    .normal()
                    .program("flatpak")
                    .args(&["uninstall", "-y", BOXBUDDY_FLATPAK])
                    .description("Removing BoxBuddy GUI...")
                    .build(),
            );
        }

        let pkgs = removable_packages(&["distrobox"]);
        if !pkgs.is_empty() {
            commands = commands.then(
                Command::builder()
                    .aur()
                    .args(&["-Rns", "--noconfirm", "distrobox"])
                    .description("Removing DistroBox and dependencies...")
                    .build(),
            );
        }

        task_runner::run(
            window_clone.upcast_ref(),
            commands.build(),
            "DistroBox Uninstall",
        );
    });

    (btn_install, btn_uninstall)
}

// ═══════════════════════════════════════════════════════════════════════════════
//  KVM / QEMU / virt-manager
// ═══════════════════════════════════════════════════════════════════════════════

/// Explicit package list replacing `virt-manager-meta` (XeroLinux-specific).
///
/// | Package        | Purpose                                        |
/// |----------------|------------------------------------------------|
/// | qemu-desktop   | QEMU emulator (desktop profile, audio+display) |
/// | libvirt        | Virtualization API daemon                      |
/// | virt-manager   | GTK GUI for managing VMs                       |
/// | virt-viewer    | Remote VM display client (SPICE/VNC)           |
/// | edk2-ovmf      | UEFI firmware for VMs                          |
/// | dnsmasq        | NAT/DHCP networking for libvirt                |
/// | iptables-nft   | Firewall backend for libvirt networking        |
/// | openbsd-netcat | Network utility (replaces gnu-netcat)          |
/// | swtpm          | Software TPM 2.0 (needed for Windows 11 VMs)  |
const KVM_PACKAGES: &[&str] = &[
    "qemu-desktop",
    "libvirt",
    "virt-manager",
    "virt-viewer",
    "edk2-ovmf",
    "dnsmasq",
    "iptables-nft",
    "openbsd-netcat",
    "swtpm",
];

/// Detect CPU vendor and return the correct modprobe option for nested
/// virtualisation. Intel → `kvm-intel`, AMD → `kvm-amd`.
fn detect_kvm_nested_conf() -> (&'static str, &'static str) {
    let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").unwrap_or_default();

    if cpuinfo.contains("GenuineIntel") {
        ("kvm-intel", "options kvm-intel nested=1")
    } else {
        // AMD or fallback — kvm-amd also covers most other x86 cases
        ("kvm-amd", "options kvm-amd nested=1")
    }
}

fn setup_kvm(builder: &Builder, window: &ApplicationWindow) -> (Button, Button) {
    let btn_install = extract_widget::<Button>(builder, "btn_kvm");
    let btn_uninstall = extract_widget::<Button>(builder, "btn_kvm_uninstall");

    // ── Install ──────────────────────────────────────────────────────────
    let window_clone = window.clone();
    btn_install.connect_clicked(move |_| {
        info!("KVM install button clicked");

        let user = crate::config::env::get().user.clone();
        let (kvm_module, kvm_option) = detect_kvm_nested_conf();
        let conf_path = format!("/etc/modprobe.d/{}.conf", kvm_module);
        let write_cmd = format!("echo '{}' > {}", kvm_option, conf_path);

        let mut commands = CommandSequence::new();

        // Resolve iptables / netcat conflicts safely.
        // iptables (legacy) conflicts with iptables-nft; gnu-netcat conflicts
        // with openbsd-netcat. Only act when the conflicting variant is present,
        // exit 0 regardless so the sequence continues.
        commands = commands.then(
            Command::builder()
                .privileged()
                .program("sh")
                .args(&[
                    "-c",
                    "pacman -Qi iptables &>/dev/null && \
                     ! pacman -Qi iptables-nft &>/dev/null && \
                     pacman -Rdd --noconfirm iptables || true; \
                     pacman -Qi gnu-netcat &>/dev/null && \
                     pacman -Rdd --noconfirm gnu-netcat || true",
                ])
                .description("Resolving package conflicts if needed...")
                .build(),
        );

        // Install all packages explicitly (no meta-package).
        commands = commands.then(
            Command::builder()
                .aur()
                .args(&[
                    "-S", "--noconfirm", "--needed",
                    "qemu-desktop",
                    "libvirt",
                    "virt-manager",
                    "virt-viewer",
                    "edk2-ovmf",
                    "dnsmasq",
                    "iptables-nft",
                    "openbsd-netcat",
                    "swtpm",
                ])
                .description("Installing virtualization packages...")
                .build(),
        );

        // Add user to libvirt group for unprivileged VM management.
        commands = commands
            .then(
                Command::builder()
                    .privileged()
                    .program("usermod")
                    .args(&["-aG", "libvirt", &user])
                    .description("Adding your user to libvirt group...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("sh")
                    .args(&["-c", &write_cmd])
                    .description("Enabling nested virtualization...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["enable", "--now", "libvirtd.service"])
                    .description("Enabling libvirtd service...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["restart", "libvirtd.service"])
                    .description("Restarting libvirtd service...")
                    .build(),
            );

        task_runner::run(window_clone.upcast_ref(), commands.build(), "KVM / QEMU Setup");
    });

    // ── Uninstall ────────────────────────────────────────────────────────
    let window_clone = window.clone();
    btn_uninstall.connect_clicked(move |_| {
        info!("KVM uninstall button clicked");

        let user = crate::config::env::get().user.clone();
        let pkgs = removable_packages(KVM_PACKAGES);

        let mut commands = CommandSequence::new()
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["stop", "libvirtd.service", "libvirtd.socket", "libvirtd-ro.socket"])
                    .description("Stopping libvirtd services...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["disable", "libvirtd.service", "libvirtd.socket", "libvirtd-ro.socket"])
                    .description("Disabling libvirtd services...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("gpasswd")
                    .args(&["-d", &user, "libvirt"])
                    .description("Removing your user from libvirt group...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("rm")
                    .args(&[
                        "-f",
                        "/etc/modprobe.d/kvm-intel.conf",
                        "/etc/modprobe.d/kvm-amd.conf",
                    ])
                    .description("Removing nested virtualization config...")
                    .build(),
            );

        if !pkgs.is_empty() {
            let mut args = vec!["-Rns".to_string(), "--noconfirm".to_string()];
            args.extend(pkgs);
            let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
            commands = commands.then(
                Command::builder()
                    .aur()
                    .args(&refs)
                    .description("Removing virtualization packages and dependencies...")
                    .build(),
            );
        }

        task_runner::run(
            window_clone.upcast_ref(),
            commands.build(),
            "KVM / QEMU Uninstall",
        );
    });

    (btn_install, btn_uninstall)
}

// ═══════════════════════════════════════════════════════════════════════════════
//  iOS iPA Sideloader (Plume Impactor)
// ═══════════════════════════════════════════════════════════════════════════════

const PLUME_FLATPAK: &str = "dev.khcrysalis.PlumeImpactor";

fn setup_ipa_sideloader(builder: &Builder, window: &ApplicationWindow) -> (Button, Button) {
    let btn_install = extract_widget::<Button>(builder, "btn_ipa_sideloader");
    let btn_uninstall = extract_widget::<Button>(builder, "btn_ipa_sideloader_uninstall");

    // ── Install ──────────────────────────────────────────────────────────
    let window_clone = window.clone();
    btn_install.connect_clicked(move |_| {
        info!("iOS iPA Sideloader install button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .normal()
                    .program("flatpak")
                    .args(&["install", "-y", "flathub", PLUME_FLATPAK])
                    .description("Installing Plume Impactor from Flathub...")
                    .build(),
            )
            .build();

        task_runner::run(window_clone.upcast_ref(), commands, "iOS iPA Sideloader Setup");
    });

    // ── Uninstall ────────────────────────────────────────────────────────
    let window_clone = window.clone();
    btn_uninstall.connect_clicked(move |_| {
        info!("iOS iPA Sideloader uninstall button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .normal()
                    .program("flatpak")
                    .args(&["uninstall", "-y", PLUME_FLATPAK])
                    .description("Removing Plume Impactor...")
                    .build(),
            )
            .build();

        task_runner::run(
            window_clone.upcast_ref(),
            commands,
            "iOS iPA Sideloader Uninstall",
        );
    });

    (btn_install, btn_uninstall)
}
