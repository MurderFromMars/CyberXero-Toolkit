//! Containers and VMs page button handlers.
//!
//! Handles:
//! - Docker installation and setup
//! - Podman installation (with optional Desktop)
//! - VirtualBox installation
//! - DistroBox installation
//! - KVM/QEMU virtualization setup

use crate::core;
use crate::ui::selection_dialog;
use crate::ui::terminal;
use crate::utils;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Label};
use log::{info, warn};
use vte4::prelude::*;
use vte4::Terminal;

/// Set up all button handlers for the containers/VMs page
pub fn setup_handlers(page_builder: &Builder, main_builder: &Builder) {
    let terminal_output: Terminal = main_builder
        .object("global_terminal_output_view")
        .expect("Failed to get terminal output view");
    let terminal_title: Label = main_builder
        .object("global_terminal_title")
        .expect("Failed to get terminal title label");

    setup_docker(&page_builder, &terminal_output, &terminal_title);
    setup_podman(&page_builder, &terminal_output, &terminal_title);
    setup_vbox(&page_builder, &terminal_output, &terminal_title);
    setup_distrobox(&page_builder, &terminal_output, &terminal_title);
    setup_kvm(&page_builder, &terminal_output, &terminal_title);
}

fn setup_docker(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_docker) = page_builder.object::<gtk4::Button>("btn_docker") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_docker.connect_clicked(move |button| {
            info!("Containers/VMs: Docker button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(b"\r\nAnother action is already running. Please wait for it to complete.\r\n");
                return;
            }

            let helper = match utils::detect_aur_helper() {
                Some(h) => h,
                None => {
                    warn!("No AUR helper detected");
                    terminal_clone.feed(b"\r\nERROR: No AUR helper detected (paru or yay required).\r\n");
                    return;
                }
            };

            let commands = vec![
                terminal::TerminalCommand::new(helper, &["-S", "--noconfirm", "--needed",
                    "docker", "docker-compose", "docker-buildx"]),
                terminal::TerminalCommand::new("sudo", &["systemctl", "enable", "--now", "docker.service"]),
                terminal::TerminalCommand::new("sudo", &["groupadd", "-f", "docker"]),
                terminal::TerminalCommand::new("sudo", &["usermod", "-aG", "docker", &std::env::var("USER").unwrap_or_else(|_| "user".to_string())]),
            ];

            terminal::run_terminal_commands(
                button,
                &terminal_clone,
                &title_clone,
                commands,
                "Docker Installation & Setup",
            );
        });
    }
}

fn setup_podman(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_podman) = page_builder.object::<gtk4::Button>("btn_podman") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_podman.connect_clicked(move |button| {
            info!("Containers/VMs: Podman button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(b"\r\nAnother action is already running. Please wait for it to complete.\r\n");
                return;
            }

            let widget = button.clone().upcast::<gtk4::Widget>();
            let window = widget.root()
                .and_then(|root| root.downcast::<ApplicationWindow>().ok());

            if let Some(window) = window {
                let terminal_for_dialog = terminal_clone.clone();
                let title_for_dialog = title_clone.clone();
                let button_for_dialog = button.clone();
                let window_ref = window.upcast_ref::<gtk4::Window>();

                let config = selection_dialog::SelectionDialogConfig::new(
                    "Podman Installation",
                    "Podman will be installed. Would you also like to install Podman Desktop GUI?",
                )
                .add_option(selection_dialog::SelectionOption::new(
                    "podman_desktop",
                    "Podman Desktop",
                    "GUI for managing containers (optional)",
                    core::is_flatpak_installed("io.podman_desktop.PodmanDesktop"),
                ))
                .confirm_label("Install");

                selection_dialog::show_selection_dialog(
                    window_ref,
                    config,
                    move |selected_ids| {
                        let helper = match utils::detect_aur_helper() {
                            Some(h) => h,
                            None => {
                                warn!("No AUR helper detected");
                                terminal_for_dialog.feed(b"\r\nERROR: No AUR helper detected (paru or yay required).\r\n");
                                return;
                            }
                        };

                        let mut commands = vec![
                            terminal::TerminalCommand::new(helper, &["-S", "--noconfirm", "--needed",
                                "podman", "podman-docker"]),
                            terminal::TerminalCommand::new("sudo", &["systemctl", "enable", "--now", "podman.socket"]),
                        ];

                        if selected_ids.contains(&"podman_desktop".to_string()) {
                            commands.push(terminal::TerminalCommand::new("flatpak",
                                &["install", "-y", "flathub", "io.podman_desktop.PodmanDesktop"]));
                        }

                        terminal::run_terminal_commands(
                            &button_for_dialog,
                            &terminal_for_dialog,
                            &title_for_dialog,
                            commands,
                            "Podman Installation & Setup",
                        );
                    },
                );
            }
        });
    }
}

fn setup_vbox(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_vbox) = page_builder.object::<gtk4::Button>("btn_vbox") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_vbox.connect_clicked(move |button| {
            info!("Containers/VMs: vBox button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(b"\r\nAnother action is already running. Please wait for it to complete.\r\n");
                return;
            }

            let helper = match utils::detect_aur_helper() {
                Some(h) => h,
                None => {
                    warn!("No AUR helper detected");
                    terminal_clone.feed(b"\r\nERROR: No AUR helper detected (paru or yay required).\r\n");
                    return;
                }
            };

            let commands = vec![
                terminal::TerminalCommand::new(helper, &["-S", "--noconfirm", "--needed", "virtualbox-meta"]),
            ];

            terminal::run_terminal_commands(
                button,
                &terminal_clone,
                &title_clone,
                commands,
                "VirtualBox Installation",
            );
        });
    }
}

fn setup_distrobox(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_distrobox) = page_builder.object::<gtk4::Button>("btn_distrobox") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_distrobox.connect_clicked(move |button| {
            info!("Containers/VMs: DistroBox button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(b"\r\nAnother action is already running. Please wait for it to complete.\r\n");
                return;
            }

            let helper = match utils::detect_aur_helper() {
                Some(h) => h,
                None => {
                    warn!("No AUR helper detected");
                    terminal_clone.feed(b"\r\nERROR: No AUR helper detected (paru or yay required).\r\n");
                    return;
                }
            };

            let commands = vec![
                terminal::TerminalCommand::new(helper, &["-S", "--noconfirm", "--needed", "distrobox"]),
                terminal::TerminalCommand::new("flatpak", &["install", "-y", "io.github.dvlv.boxbuddyrs"]),
            ];

            terminal::run_terminal_commands(
                button,
                &terminal_clone,
                &title_clone,
                commands,
                "Distrobox Installation",
            );
        });
    }
}

fn setup_kvm(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_kvm) = page_builder.object::<gtk4::Button>("btn_kvm") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_kvm.connect_clicked(move |button| {
            info!("Containers/VMs: KVM button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(b"\r\nAnother action is already running. Please wait for it to complete.\r\n");
                return;
            }

            let helper = match utils::detect_aur_helper() {
                Some(h) => h,
                None => {
                    warn!("No AUR helper detected");
                    terminal_clone.feed(b"\r\nERROR: No AUR helper detected (paru or yay required).\r\n");
                    return;
                }
            };

            let mut commands = vec![];

            if core::is_package_installed("iptables") {
                commands.push(terminal::TerminalCommand::new(helper,
                    &["-Rdd", "--noconfirm", "iptables"]));
            }
            if core::is_package_installed("gnu-netcat") {
                commands.push(terminal::TerminalCommand::new(helper,
                    &["-Rdd", "--noconfirm", "gnu-netcat"]));
            }

            commands.push(terminal::TerminalCommand::new(helper,
                &["-S", "--noconfirm", "--needed", "virt-manager-meta", "openbsd-netcat"]));
            commands.push(terminal::TerminalCommand::new("sudo",
                &["sh", "-c", "echo 'options kvm-intel nested=1' > /etc/modprobe.d/kvm-intel.conf"]));
            commands.push(terminal::TerminalCommand::new("sudo",
                &["systemctl", "restart", "libvirtd.service"]));

            terminal::run_terminal_commands(
                button,
                &terminal_clone,
                &title_clone,
                commands,
                "KVM/QEMU Virtualization Setup",
            );
        });
    }
}
