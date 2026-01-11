//! Containers and VMs page button handlers.
//!
//! Handles:
//! - Docker installation and setup
//! - Podman installation (with optional Desktop)
//! - VirtualBox installation
//! - DistroBox installation
//! - KVM/QEMU virtualization setup
//! - iOS iPA Sideloader (Plume Impactor)

use crate::core;
use crate::ui::dialogs::selection::{
    show_selection_dialog, SelectionDialogConfig, SelectionOption, SelectionType,
};
use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::extract_widget;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button};
use log::info;

/// Set up all button handlers for the containers/VMs page.
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    setup_docker(page_builder, window);
    setup_podman(page_builder, window);
    setup_vbox(page_builder, window);
    setup_distrobox(page_builder, window);
    setup_kvm(page_builder, window);
    setup_ipa_sideloader(page_builder, window);
}

fn setup_docker(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_docker");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Docker button clicked");

        let user = crate::config::env::get().user.clone();

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&[
                        "-S",
                        "--noconfirm",
                        "--needed",
                        "docker",
                        "docker-compose",
                        "docker-buildx",
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

        task_runner::run(window.upcast_ref(), commands, "Docker Setup");
    });
}

fn setup_podman(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_podman");
    let window = window.clone();
    button.connect_clicked(move |_| {
        info!("Podman button clicked");

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
            core::is_flatpak_installed("io.podman_desktop.PodmanDesktop"),
        ))
        .confirm_label("Install");

        let window_for_closure = window.clone();
        show_selection_dialog(window.upcast_ref(), config, move |selected| {
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
                        .args(&[
                            "install",
                            "-y",
                            "flathub",
                            "io.podman_desktop.PodmanDesktop",
                        ])
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
}

fn setup_vbox(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_vbox");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("VirtualBox button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&["-S", "--noconfirm", "--needed", "virtualbox-meta"])
                    .description("Installing VirtualBox...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "VirtualBox Setup");
    });
}

fn setup_distrobox(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_distrobox");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("DistroBox button clicked");

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
                    .args(&["install", "-y", "io.github.dvlv.boxbuddyrs"])
                    .description("Installing BoxBuddy GUI...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "DistroBox Setup");
    });
}

fn setup_kvm(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_kvm");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("KVM button clicked");

        let mut commands = CommandSequence::new();

        // Remove conflicting packages if installed
        if core::is_package_installed("iptables") {
            commands = commands.then(
                Command::builder()
                    .aur()
                    .args(&["-Rdd", "--noconfirm", "iptables"])
                    .description("Removing conflicting iptables...")
                    .build(),
            );
        }

        if core::is_package_installed("gnu-netcat") {
            commands = commands.then(
                Command::builder()
                    .aur()
                    .args(&["-Rdd", "--noconfirm", "gnu-netcat"])
                    .description("Removing conflicting gnu-netcat...")
                    .build(),
            );
        }

        commands = commands.then(
            Command::builder()
                .aur()
                .args(&[
                    "-S",
                    "--noconfirm",
                    "--needed",
                    "virt-manager-meta",
                    "openbsd-netcat",
                ])
                .description("Installing virtualization packages...")
                .build(),
        );

        commands = commands.then(
            Command::builder()
                .privileged()
                .program("sh")
                .args(&[
                    "-c",
                    "echo 'options kvm-intel nested=1' > /etc/modprobe.d/kvm-intel.conf",
                ])
                .description("Enabling nested virtualization...")
                .build(),
        );

        commands = commands.then(
            Command::builder()
                .privileged()
                .program("systemctl")
                .args(&["restart", "libvirtd.service"])
                .description("Restarting libvirtd service...")
                .build(),
        );

        task_runner::run(window.upcast_ref(), commands.build(), "KVM / QEMU Setup");
    });
}

fn setup_ipa_sideloader(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_ipa_sideloader");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("iOS iPA Sideloader button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&["-S", "--noconfirm", "--needed", "plume-impactor"])
                    .description("Installing Plume Impactor...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "iOS iPA Sideloader Setup");
    });
}
