//! Gaming tools page button handlers.
//!
//! Handles:
//! - Steam AiO installation
//! - Game controller drivers
//! - Gamescope configuration
//! - LACT GPU overclocking
//! - Game launchers (Lutris, Heroic, Bottles)

use crate::core;
use crate::ui::selection_dialog;
use crate::ui::terminal;
use crate::utils;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Label};
use log::{info, warn};
use vte4::prelude::*;
use vte4::Terminal;

/// Set up all button handlers for the gaming tools page
pub fn setup_handlers(page_builder: &Builder, main_builder: &Builder) {
    let terminal_output: Terminal = main_builder
        .object("global_terminal_output_view")
        .expect("Failed to get terminal output view");
    let terminal_title: Label = main_builder
        .object("global_terminal_title")
        .expect("Failed to get terminal title label");

    setup_steam_aio(&page_builder, &terminal_output, &terminal_title);
    setup_controllers(&page_builder, &terminal_output, &terminal_title);
    setup_gamescope_cfg(&page_builder);
    setup_lact_oc(&page_builder, &terminal_output, &terminal_title);
    setup_lutris(&page_builder, &terminal_output, &terminal_title);
    setup_heroic(&page_builder, &terminal_output, &terminal_title);
    setup_bottles(&page_builder, &terminal_output, &terminal_title);
}

fn setup_steam_aio(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_steam_aio) = page_builder.object::<gtk4::Button>("btn_steam_aio") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_steam_aio.connect_clicked(move |button| {
            info!("Gaming tools: Steam AiO button clicked");

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
                  "steam", "lib32-pipewire-jack", "gamemode", "gamescope",
                  "mangohud", "mangoverlay", "lib32-mangohud", "wine-meta",
                  "wine-nine", "ttf-liberation", "lib32-fontconfig", "wqy-zenhei",
                  "vkd3d", "giflib", "lib32-giflib", "libpng", "lib32-libpng",
                  "libldap", "lib32-libldap", "gnutls", "lib32-gnutls",
                  "mpg123", "lib32-mpg123", "openal", "lib32-openal",
                  "v4l-utils", "lib32-v4l-utils", "libpulse", "lib32-libpulse",
                  "libgpg-error", "lib32-libgpg-error", "alsa-plugins", "lib32-alsa-plugins",
                  "alsa-lib", "lib32-alsa-lib", "libjpeg-turbo", "lib32-libjpeg-turbo",
                  "sqlite", "lib32-sqlite", "libxcomposite", "lib32-libxcomposite",
                  "libxinerama", "lib32-libgcrypt", "libgcrypt", "lib32-libxinerama",
                  "ncurses", "lib32-ncurses", "ocl-icd", "lib32-ocl-icd",
                  "libxslt", "lib32-libxslt", "libva", "lib32-libva",
                  "gtk3", "lib32-gtk3", "gst-plugins-base-libs", "lib32-gst-plugins-base-libs",
                  "vulkan-icd-loader", "lib32-vulkan-icd-loader", "cups", "dosbox",
                  "lib32-opencl-icd-loader", "lib32-vkd3d", "opencl-icd-loader"]),
            ];

            terminal::run_terminal_commands(
                button,
                &terminal_clone,
                &title_clone,
                commands,
                "Steam AiO Installation",
            );
        });
    }
}

fn setup_controllers(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_controllers) = page_builder.object::<gtk4::Button>("btn_controllers") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_controllers.connect_clicked(move |button| {
            info!("Gaming tools: Controllers button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(b"\r\nAnother action is already running. Please wait for it to complete.\r\n");
                return;
            }

            let dualsense_installed = core::is_package_installed("dualsensectl");
            let dualshock4_installed = core::is_package_installed("ds4drv");
            let xboxone_installed = core::is_package_installed("xone-dkms");

            let widget = button.clone().upcast::<gtk4::Widget>();
            let window = widget.root()
                .and_then(|root| root.downcast::<ApplicationWindow>().ok());

            if let Some(window) = window {
                let terminal_for_dialog = terminal_clone.clone();
                let title_for_dialog = title_clone.clone();
                let button_for_dialog = button.clone();
                let window_ref = window.upcast_ref::<gtk4::Window>();

                let config = selection_dialog::SelectionDialogConfig::new(
                    "Game Controller Drivers",
                    "Select which controller drivers to install. Already installed drivers are not shown.",
                )
                .add_option(selection_dialog::SelectionOption::new(
                    "dualsense",
                    "DualSense Controller",
                    "PlayStation 5 DualSense controller driver",
                    dualsense_installed,
                ))
                .add_option(selection_dialog::SelectionOption::new(
                    "dualshock4",
                    "DualShock 4 Controller",
                    "PlayStation 4 DualShock 4 controller driver",
                    dualshock4_installed,
                ))
                .add_option(selection_dialog::SelectionOption::new(
                    "xboxone",
                    "Xbox One Controller",
                    "Xbox One wireless controller driver",
                    xboxone_installed,
                ))
                .confirm_label("Install");

                selection_dialog::show_selection_dialog(
                    window_ref,
                    config,
                    move |selected_ids| {
                        info!("Selected controllers: {:?}", selected_ids);

                        if selected_ids.is_empty() {
                            return;
                        }

                        let helper = match utils::detect_aur_helper() {
                            Some(h) => h,
                            None => {
                                warn!("No AUR helper detected");
                                terminal_for_dialog.feed(b"\r\nERROR: No AUR helper detected (paru or yay required).\r\n");
                                return;
                            }
                        };

                        let mut commands = Vec::new();

                        for id in selected_ids {
                            match id.as_str() {
                                "dualsense" => {
                                    commands.push(terminal::TerminalCommand::new(
                                        helper,
                                        &["-S", "--noconfirm", "--needed", "dualsensectl", "game-devices-udev"],
                                    ));
                                }
                                "dualshock4" => {
                                    commands.push(terminal::TerminalCommand::new(
                                        helper,
                                        &["-S", "--noconfirm", "--needed", "ds4drv", "game-devices-udev"],
                                    ));
                                }
                                "xboxone" => {
                                    commands.push(terminal::TerminalCommand::new(
                                        helper,
                                        &["-S", "--noconfirm", "--needed", "xone-dkms", "game-devices-udev"],
                                    ));
                                }
                                _ => {}
                            }
                        }

                        if !commands.is_empty() {
                            terminal::run_terminal_commands(
                                &button_for_dialog,
                                &terminal_for_dialog,
                                &title_for_dialog,
                                commands,
                                "Game Controller Drivers Installation",
                            );
                        }
                    },
                );
            }
        });
    }
}

fn setup_gamescope_cfg(page_builder: &Builder) {
    if let Some(btn_gamescope_cfg) = page_builder.object::<gtk4::Button>("btn_gamescope_cfg") {
        btn_gamescope_cfg.connect_clicked(move |_| {
            info!("Gaming tools: Gamescope CFG button clicked - opening gamescope-gui");
            let _ = std::process::Command::new("xdg-open")
                .arg("https://sidewalksndskeletons.github.io/gamescope-gui/")
                .spawn();
        });
    }
}

fn setup_lact_oc(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_lact_oc) = page_builder.object::<gtk4::Button>("btn_lact_oc") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_lact_oc.connect_clicked(move |button| {
            info!("Gaming tools: LACT OC button clicked");

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
                terminal::TerminalCommand::new(helper, &["-S", "--noconfirm", "--needed", "lact"]),
                terminal::TerminalCommand::new("sudo", &["systemctl", "enable", "--now", "lactd"]),
            ];

            terminal::run_terminal_commands(
                button,
                &terminal_clone,
                &title_clone,
                commands,
                "LACT GPU OC Installation & Setup",
            );
        });
    }
}

fn setup_lutris(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_lutris) = page_builder.object::<gtk4::Button>("btn_lutris") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_lutris.connect_clicked(move |button| {
            info!("Gaming tools: Lutris button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(b"\r\nAnother action is already running. Please wait for it to complete.\r\n");
                return;
            }

            let commands = vec![
                terminal::TerminalCommand::new("flatpak", &["install", "-y", "net.lutris.Lutris",
                  "org.freedesktop.Platform.VulkanLayer.gamescope/x86_64/24.08",
                  "org.freedesktop.Platform.VulkanLayer.MangoHud"]),
            ];

            terminal::run_terminal_commands(
                button,
                &terminal_clone,
                &title_clone,
                commands,
                "Lutris Installation",
            );
        });
    }
}

fn setup_heroic(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_heroic) = page_builder.object::<gtk4::Button>("btn_heroic") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_heroic.connect_clicked(move |button| {
            info!("Gaming tools: Heroic button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(b"\r\nAnother action is already running. Please wait for it to complete.\r\n");
                return;
            }

            let commands = vec![
                terminal::TerminalCommand::new("flatpak", &["install", "-y", "com.heroicgameslauncher.hgl",
                  "org.freedesktop.Platform.VulkanLayer.gamescope/x86_64/24.08",
                  "org.freedesktop.Platform.VulkanLayer.MangoHud"]),
            ];

            terminal::run_terminal_commands(
                button,
                &terminal_clone,
                &title_clone,
                commands,
                "Heroic Games Launcher Installation",
            );
        });
    }
}

fn setup_bottles(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_bottles) = page_builder.object::<gtk4::Button>("btn_bottles") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_bottles.connect_clicked(move |button| {
            info!("Gaming tools: Bottles button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(b"\r\nAnother action is already running. Please wait for it to complete.\r\n");
                return;
            }

            let commands = vec![
                terminal::TerminalCommand::new("flatpak", &["install", "-y", "com.usebottles.bottles",
                  "org.freedesktop.Platform.VulkanLayer.gamescope",
                  "org.freedesktop.Platform.VulkanLayer.MangoHud"]),
            ];

            terminal::run_terminal_commands(
                button,
                &terminal_clone,
                &title_clone,
                commands,
                "Bottles Installation",
            );
        });
    }
}

