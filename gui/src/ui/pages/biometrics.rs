//! Biometrics page button handlers.
//!
//! Handles:
//! - Fingerprint reader setup (xfprintd-gui)
//! - Howdy facial recognition setup (xero-howdy-qt - build from source)

use crate::core;
use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::extract_widget;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder};
use log::{error, info};
use std::process::{Command as StdCommand, Stdio};

/// Set up all button handlers for the biometrics page
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    setup_fingerprint(page_builder, window);
    setup_howdy(page_builder, window);
}

/// Helper to update button appearance based on installation status
fn update_button_state(
    setup_button: &gtk4::Button,
    uninstall_button: &gtk4::Button,
    is_installed: bool,
) {
    if is_installed {
        setup_button.set_label("Launch App");
        setup_button.add_css_class("suggested-action");
        uninstall_button.set_visible(true);
    } else {
        setup_button.set_label("Install");
        setup_button.remove_css_class("suggested-action");
        uninstall_button.set_visible(false);
    }
}

/// Check if howdy is installed (either howdy-bin or howdy-git)
fn is_howdy_installed() -> bool {
    core::is_package_installed("howdy-bin") || core::is_package_installed("howdy-git")
}

/// Check if howdy-bin is available in system repos (not AUR)
fn is_howdy_bin_in_repos() -> bool {
    let output = StdCommand::new("pacman")
        .args(&["-Si", "howdy-bin"])
        .output();
    
    match output {
        Ok(result) => result.status.success(),
        Err(_) => false,
    }
}

fn setup_fingerprint(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_fingerprint_setup =
        extract_widget::<gtk4::Button>(page_builder, "btn_fingerprint_setup");
    let btn_fingerprint_uninstall =
        extract_widget::<gtk4::Button>(page_builder, "btn_fingerprint_uninstall");

    // Initial check
    let is_installed = core::is_package_installed("xfprintd-gui");
    update_button_state(&btn_fingerprint_setup, &btn_fingerprint_uninstall, is_installed);

    // Update on window focus (e.g. after installation completes)
    let btn_setup_clone = btn_fingerprint_setup.clone();
    let btn_uninstall_clone = btn_fingerprint_uninstall.clone();
    window.connect_is_active_notify(move |window| {
        if window.is_active() {
            let is_installed = core::is_package_installed("xfprintd-gui");
            update_button_state(&btn_setup_clone, &btn_uninstall_clone, is_installed);
        }
    });

    // Setup/Launch button handler
    let window_clone = window.clone();
    btn_fingerprint_setup.connect_clicked(move |_| {
        info!("Biometrics: Fingerprint setup button clicked");

        // Check again at click time
        if core::is_package_installed("xfprintd-gui") {
            info!("Launching xfprintd-gui...");
            if let Err(e) = StdCommand::new("xfprintd-gui")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                error!("Failed to launch xfprintd-gui: {}", e);
            }
        } else {
            let commands = CommandSequence::new()
                .then(
                    Command::builder()
                        .aur()
                        .args(&["-S", "--noconfirm", "--needed", "xfprintd-gui"])
                        .description("Installing Fingerprint GUI Tool...")
                        .build(),
                )
                .build();

            task_runner::run(
                window_clone.upcast_ref(),
                commands,
                "Install Fingerprint GUI Tool",
            );
        }
    });

    // Uninstall button handler
    let window_clone = window.clone();
    btn_fingerprint_uninstall.connect_clicked(move |_| {
        info!("Biometrics: Fingerprint uninstall button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&["-Rns", "--noconfirm", "xfprintd-gui"])
                    .description("Uninstalling Fingerprint GUI Tool...")
                    .build(),
            )
            .build();

        task_runner::run(
            window_clone.upcast_ref(),
            commands,
            "Uninstall Fingerprint GUI Tool",
        );
    });
}

fn setup_howdy(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_howdy_setup = extract_widget::<gtk4::Button>(page_builder, "btn_howdy_setup");
    let btn_howdy_uninstall = extract_widget::<gtk4::Button>(page_builder, "btn_howdy_uninstall");

    // Initial check - check if binary exists instead of package
    let is_installed = std::path::Path::new("/usr/bin/xero-howdy-qt").exists();
    update_button_state(&btn_howdy_setup, &btn_howdy_uninstall, is_installed);

    // Update on window focus (e.g. after installation completes)
    let btn_setup_clone = btn_howdy_setup.clone();
    let btn_uninstall_clone = btn_howdy_uninstall.clone();
    window.connect_is_active_notify(move |window| {
        if window.is_active() {
            let is_installed = std::path::Path::new("/usr/bin/xero-howdy-qt").exists();
            update_button_state(&btn_setup_clone, &btn_uninstall_clone, is_installed);
        }
    });

    // Setup/Launch button handler
    let window_clone = window.clone();
    btn_howdy_setup.connect_clicked(move |_| {
        info!("Biometrics: Howdy setup button clicked");

        // Check again at click time - check if binary exists instead of package
        if std::path::Path::new("/usr/bin/xero-howdy-qt").exists() {
            info!("Launching xero-howdy-qt...");
            if let Err(e) = StdCommand::new("xero-howdy-qt")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                error!("Failed to launch xero-howdy-qt: {}", e);
            }
        } else {
            // Determine which howdy package to install
            let howdy_package = if is_howdy_installed() {
                info!("Howdy already installed, skipping installation");
                None
            } else if is_howdy_bin_in_repos() {
                info!("howdy-bin found in repos, will install from there");
                Some("howdy-bin")
            } else {
                info!("howdy-bin not in repos, will install howdy-git from AUR");
                Some("howdy-git")
            };

            // Build and install Howdy Qt from source
            let mut commands = CommandSequence::new();

            // Install dependencies including the appropriate howdy package
            if let Some(howdy_pkg) = howdy_package {
                commands = commands.then(
                    Command::builder()
                        .aur()
                        .args(&["-S", "--noconfirm", "--needed", "rust", "cargo", "clang", "qt6-base", "qt6-declarative", howdy_pkg])
                        .description("Installing build dependencies and Howdy...")
                        .build(),
                );
            } else {
                // Howdy already installed, just install build dependencies
                commands = commands.then(
                    Command::builder()
                        .aur()
                        .args(&["-S", "--noconfirm", "--needed", "rust", "cargo", "clang", "qt6-base", "qt6-declarative"])
                        .description("Installing build dependencies...")
                        .build(),
                );
            }

            commands = commands
                .then(
                    Command::builder()
                        .normal()
                        .program("sh")
                        .args(&[
                            "-c",
                            "rm -rf /tmp/xero-howdy-qt && git clone https://github.com/XeroLinuxDev/xero-howdy-qt.git /tmp/xero-howdy-qt",
                        ])
                        .description("Cloning Howdy Qt repository...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .normal()
                        .program("sh")
                        .args(&[
                            "-c",
                            "cd /tmp/xero-howdy-qt && cargo build --release",
                        ])
                        .description("Building Howdy Qt (this may take a few minutes)...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .privileged()
                        .program("sh")
                        .args(&[
                            "-c",
                            "install -Dm755 /tmp/xero-howdy-qt/target/release/xero-howdy-qt /usr/bin/xero-howdy-qt",
                        ])
                        .description("Installing Howdy Qt to system...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .normal()
                        .program("rm")
                        .args(&["-rf", "/tmp/xero-howdy-qt"])
                        .description("Cleaning up build directory...")
                        .build(),
                )
                .build();

            task_runner::run(window_clone.upcast_ref(), commands, "Install Howdy Qt (Build from Source)");
        }
    });

    // Uninstall button handler
    let window_clone = window.clone();
    btn_howdy_uninstall.connect_clicked(move |_| {
        info!("Biometrics: Howdy uninstall button clicked");

        // Build uninstall commands - remove binary and whichever howdy package is installed
        let mut commands = CommandSequence::new()
            .then(
                Command::builder()
                    .privileged()
                    .program("rm")
                    .args(&["-f", "/usr/bin/xero-howdy-qt"])
                    .description("Removing Howdy Qt binary...")
                    .build(),
            );

        // Remove whichever howdy package is installed
        if core::is_package_installed("howdy-bin") {
            commands = commands.then(
                Command::builder()
                    .aur()
                    .args(&["-Rns", "--noconfirm", "howdy-bin"])
                    .description("Uninstalling Howdy (howdy-bin)...")
                    .build(),
            );
        } else if core::is_package_installed("howdy-git") {
            commands = commands.then(
                Command::builder()
                    .aur()
                    .args(&["-Rns", "--noconfirm", "howdy-git"])
                    .description("Uninstalling Howdy (howdy-git)...")
                    .build(),
            );
        }

        let commands = commands.build();

        task_runner::run(
            window_clone.upcast_ref(),
            commands,
            "Uninstall Howdy Qt",
        );
    });
}
