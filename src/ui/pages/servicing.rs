//! Servicing and system tweaks page button handlers.
//!
//! Handles:
//! - Clear Pacman cache
//! - Unlock Pacman database
//! - Plasma X11 session installation
//! - VM guest utilities
//! - WayDroid guide
//! - Fix GPGME database
//! - Fix Arch keyring
//! - Update mirrorlist

use crate::core;
use crate::ui::selection_dialog;
use crate::ui::terminal;
use crate::utils;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Label};
use log::{info, warn};
use vte4::prelude::*;
use vte4::Terminal;

/// Set up all button handlers for the servicing/system tweaks page
pub fn setup_handlers(page_builder: &Builder, main_builder: &Builder) {
    let terminal_output: Terminal = main_builder
        .object("global_terminal_output_view")
        .expect("Failed to get terminal output view");
    let terminal_title: Label = main_builder
        .object("global_terminal_title")
        .expect("Failed to get terminal title label");

    setup_clr_pacman(&page_builder, &terminal_output, &terminal_title);
    setup_unlock_pacman(&page_builder, &terminal_output, &terminal_title);
    setup_plasma_x11(&page_builder, &terminal_output, &terminal_title);
    setup_vm_guest_utils(&page_builder, &terminal_output, &terminal_title);
    setup_waydroid_guide(&page_builder);
    setup_fix_gpgme(&page_builder, &terminal_output, &terminal_title);
    setup_fix_arch_keyring(&page_builder, &terminal_output, &terminal_title);
    setup_update_mirrorlist(&page_builder, &terminal_output, &terminal_title);
}

fn setup_clr_pacman(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_clr_pacman) = page_builder.object::<gtk4::Button>("btn_clr_pacman") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_clr_pacman.connect_clicked(move |button| {
            info!("Servicing: Clear Pacman Cache button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(b"\r\nAnother action is already running. Please wait for it to complete.\r\n");
                return;
            }

            let commands = vec![
                terminal::TerminalCommand::new("sudo", &["pacman", "-Scc"]),
            ];

            terminal::run_terminal_commands(
                button,
                &terminal_clone,
                &title_clone,
                commands,
                "Clear Pacman Cache",
            );
        });
    }
}

fn setup_unlock_pacman(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_unlock_pacman) = page_builder.object::<gtk4::Button>("btn_unlock_pacman") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_unlock_pacman.connect_clicked(move |button| {
            info!("Servicing: Unlock Pacman DB button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(b"\r\nAnother action is already running. Please wait for it to complete.\r\n");
                return;
            }

            let commands = vec![
                terminal::TerminalCommand::new("sudo", &["rm", "-f", "/var/lib/pacman/db.lck"]),
            ];

            terminal::run_terminal_commands(
                button,
                &terminal_clone,
                &title_clone,
                commands,
                "Unlock Pacman Database",
            );
        });
    }
}

fn setup_plasma_x11(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_plasma_x11) = page_builder.object::<gtk4::Button>("btn_plasma_x11") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_plasma_x11.connect_clicked(move |button| {
            info!("Servicing: Plasma X11 Session button clicked");

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
                terminal::TerminalCommand::new(helper, &["-S", "--noconfirm", "kwin-x11", "plasma-x11-session"]),
            ];

            terminal::run_terminal_commands(
                button,
                &terminal_clone,
                &title_clone,
                commands,
                "Install KDE X11 Session",
            );
        });
    }
}

fn setup_vm_guest_utils(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_vm_guest_utils) = page_builder.object::<gtk4::Button>("btn_vm_guest_utils") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_vm_guest_utils.connect_clicked(move |button| {
            info!("Servicing: VM Guest Utils button clicked");

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

            let output = std::process::Command::new("systemd-detect-virt").output();

            match output {
                Ok(result) if result.status.success() => {
                    let virt = String::from_utf8_lossy(&result.stdout).trim().to_string();

                    let commands = match virt.as_str() {
                        "oracle" => {
                            terminal_clone.feed(b"Detected VirtualBox environment\r\n");
                            vec![terminal::TerminalCommand::new(helper,
                                &["-S", "--needed", "--noconfirm", "virtualbox-guest-utils"])]
                        }
                        "kvm" => {
                            terminal_clone.feed(b"Detected KVM environment\r\n");
                            vec![terminal::TerminalCommand::new(helper,
                                &["-S", "--needed", "--noconfirm", "qemu-guest-agent", "spice-vdagent"])]
                        }
                        _ => vec![],
                    };

                    terminal::run_terminal_commands(
                        button,
                        &terminal_clone,
                        &title_clone,
                        commands,
                        "Install VM Guest Utilities",
                    );
                }
                _ => {
                    terminal_clone.feed(b"Failed to detect virtualization environment\r\n");
                }
            }
        });
    }
}

fn setup_waydroid_guide(page_builder: &Builder) {
    if let Some(btn_waydroid_guide) = page_builder.object::<gtk4::Button>("btn_waydroid_guide") {
        btn_waydroid_guide.connect_clicked(move |_| {
            info!("Servicing: WayDroid Guide button clicked - opening guide");
            let _ = std::process::Command::new("xdg-open")
                .arg("https://xerolinux.xyz/posts/waydroid-guide/")
                .spawn();
        });
    }
}

fn setup_fix_gpgme(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_fix_gpgme) = page_builder.object::<gtk4::Button>("btn_fix_gpgme") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_fix_gpgme.connect_clicked(move |button| {
            info!("Servicing: Fix GPGME Database button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(b"\r\nAnother action is already running. Please wait for it to complete.\r\n");
                return;
            }

            let commands = vec![
                terminal::TerminalCommand::new("sudo", &["rm", "-rf", "/var/lib/pacman/sync"]),
                terminal::TerminalCommand::new("sudo", &["pacman", "-Syy"]),
            ];

            terminal::run_terminal_commands(
                button,
                &terminal_clone,
                &title_clone,
                commands,
                "Fix GPGME Database Issue",
            );
        });
    }
}

fn setup_fix_arch_keyring(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_fix_arch_keyring) = page_builder.object::<gtk4::Button>("btn_fix_arch_keyring") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_fix_arch_keyring.connect_clicked(move |button| {
            info!("Servicing: Fix Arch Keyring button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(b"\r\nAnother action is already running. Please wait for it to complete.\r\n");
                return;
            }

            let commands = vec![
                terminal::TerminalCommand::new("sudo", &["rm", "-rf", "/etc/pacman.d/gnupg"]),
                terminal::TerminalCommand::new("sudo", &["pacman-key", "--init"]),
                terminal::TerminalCommand::new("sudo", &["pacman-key", "--populate"]),
                terminal::TerminalCommand::new("sudo", &["sh", "-c",
                    "echo 'keyserver hkp://keyserver.ubuntu.com:80' >> /etc/pacman.d/gnupg/gpg.conf"]),
                // installing archlinux-keyring should use pacman to ensure keyring handled correctly
                terminal::TerminalCommand::new("sudo", &["pacman", "-Syy", "--noconfirm", "archlinux-keyring"]),
            ];

            terminal::run_terminal_commands(
                button,
                &terminal_clone,
                &title_clone,
                commands,
                "Fix GnuPG Keyring",
            );
        });
    }
}

fn setup_update_mirrorlist(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_update_mirrorlist) = page_builder.object::<gtk4::Button>("btn_update_mirrorlist") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_update_mirrorlist.connect_clicked(move |button| {
            info!("Servicing: Update Mirrorlist button clicked");

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

                let rate_mirrors_installed = core::is_package_installed("rate-mirrors");

                let config = selection_dialog::SelectionDialogConfig::new(
                    "Update Mirrorlist",
                    "Select which mirrorlists to update. rate-mirrors will be installed if needed.",
                )
                .add_option(selection_dialog::SelectionOption::new(
                    "chaotic",
                    "Chaotic-AUR Mirrorlist",
                    "Also update Chaotic-AUR mirrorlist (optional)",
                    false,
                ))
                .confirm_label("Update");

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

                        let mut commands = vec![];

                        if !rate_mirrors_installed {
                            commands.push(terminal::TerminalCommand::new(helper,
                                &["-S", "--needed", "--noconfirm", "rate-mirrors"]));
                        }

                        commands.push(terminal::TerminalCommand::new("sudo",
                            &["sh", "-c", "rate-mirrors --allow-root --protocol https arch | tee /etc/pacman.d/mirrorlist"]));

                        if selected_ids.contains(&"chaotic".to_string()) {
                            commands.push(terminal::TerminalCommand::new("sudo",
                                &["sh", "-c", "rate-mirrors --allow-root --protocol https chaotic-aur | tee /etc/pacman.d/chaotic-mirrorlist"]));
                        }

                        commands.push(terminal::TerminalCommand::new("sudo", &["pacman", "-Syy"]));

                        terminal::run_terminal_commands(
                            &button_for_dialog,
                            &terminal_for_dialog,
                            &title_for_dialog,
                            commands,
                            "Update System Mirrorlist",
                        );
                    },
                );
            }
        });
    }
}
