//! Biometrics page button handlers.
//!
//! Handles:
//! - Fingerprint reader setup (xfprintd-gui - jailbroken edition from source)
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

fn setup_fingerprint(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_fingerprint_setup =
        extract_widget::<gtk4::Button>(page_builder, "btn_fingerprint_setup");
    let btn_fingerprint_uninstall =
        extract_widget::<gtk4::Button>(page_builder, "btn_fingerprint_uninstall");

    // Initial check - check if binary exists instead of package
    let is_installed = std::path::Path::new("/usr/bin/xfprintd-gui").exists();
    update_button_state(&btn_fingerprint_setup, &btn_fingerprint_uninstall, is_installed);

    // Update on window focus (e.g. after installation completes)
    let btn_setup_clone = btn_fingerprint_setup.clone();
    let btn_uninstall_clone = btn_fingerprint_uninstall.clone();
    window.connect_is_active_notify(move |window| {
        if window.is_active() {
            let is_installed = std::path::Path::new("/usr/bin/xfprintd-gui").exists();
            update_button_state(&btn_setup_clone, &btn_uninstall_clone, is_installed);
        }
    });

    // Setup/Launch button handler
    let window_clone = window.clone();
    btn_fingerprint_setup.connect_clicked(move |_| {
        info!("Biometrics: Fingerprint setup button clicked");

        // Check again at click time - check if binary exists instead of package
        if std::path::Path::new("/usr/bin/xfprintd-gui").exists() {
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
            // Build and install xfprintd-gui from jailbroken fork
            let commands = CommandSequence::new()
                .then(
                    Command::builder()
                        .aur()
                        .args(&["-S", "--noconfirm", "--needed", "rust", "cargo", "gtk4", "libadwaita", "glib2", "pkgconf", "polkit", "fprintd", "base-devel"])
                        .description("Installing build dependencies...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .normal()
                        .program("sh")
                        .args(&[
                            "-c",
                            "rm -rf /tmp/xfprintd-jailbreak && git clone https://github.com/MurderFromMars/xfprintd-gui.git /tmp/xfprintd-jailbreak",
                        ])
                        .description("Cloning XFPrintD GUI Jailbroken Edition...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .normal()
                        .program("bash")
                        .args(&[
                            "-c",
                            "cd /tmp/xfprintd-jailbreak && bash /tmp/xfprintd-jailbreak/install.sh",
                        ])
                        .description("Building and installing Fingerprint GUI (Jailbroken Edition)...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .normal()
                        .program("rm")
                        .args(&["-rf", "/tmp/xfprintd-jailbreak"])
                        .description("Cleaning up build directory...")
                        .build(),
                )
                .build();

            task_runner::run(
                window_clone.upcast_ref(),
                commands,
                "Install XFPrintD GUI (Jailbroken Edition)",
            );
        }
    });

    // Uninstall button handler
    let window_clone = window.clone();
    btn_fingerprint_uninstall.connect_clicked(move |_| {
        info!("Biometrics: Fingerprint uninstall button clicked");

        // Build uninstall commands - remove all installed files
        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .privileged()
                    .program("rm")
                    .args(&["-rf", "/opt/xfprintd-gui"])
                    .description("Removing XFPrintD GUI installation directory...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("rm")
                    .args(&["-f", "/usr/bin/xfprintd-gui"])
                    .description("Removing XFPrintD GUI binary symlink...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("rm")
                    .args(&["-f", "/usr/share/applications/xfprintd-gui.desktop"])
                    .description("Removing desktop entry...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("rm")
                    .args(&["-f", "/usr/share/icons/hicolor/scalable/apps/xfprintd-gui.svg"])
                    .description("Removing application icon...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("gtk-update-icon-cache")
                    .args(&["-q", "-t", "-f", "/usr/share/icons/hicolor"])
                    .description("Updating icon cache...")
                    .build(),
            )
            .build();

        task_runner::run(
            window_clone.upcast_ref(),
            commands,
            "Uninstall XFPrintD GUI (Jailbroken Edition)",
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
            // Build and install Howdy Qt from source
            let mut commands = CommandSequence::new();

            // First, install build dependencies from AUR helper
            commands = commands.then(
                Command::builder()
                    .aur()
                    .args(&["-S", "--noconfirm", "--needed", "rust", "cargo", "clang", "qt6-base", "qt6-declarative"])
                    .description("Installing build dependencies...")
                    .build(),
            );

            // Then install Howdy if not already installed
            if !is_howdy_installed() {
                info!("Installing howdy-git from AUR");
                commands = commands.then(
                    Command::builder()
                        .aur()
                        .args(&["-S", "--noconfirm", "--needed", "howdy-git"])
                        .description("Installing Howdy from AUR...")
                        .build(),
                );
            } else {
                info!("Howdy already installed, skipping Howdy installation");
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

        // Build uninstall commands - remove binary, howdy-git package, and python dependencies
        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .privileged()
                    .program("rm")
                    .args(&["-f", "/usr/bin/xero-howdy-qt"])
                    .description("Removing Howdy Qt binary...")
                    .build(),
            )
            .then(
                Command::builder()
                    .aur()
                    .args(&["-Rns", "--noconfirm", "howdy-git"])
                    .description("Uninstalling Howdy (howdy-git)...")
                    .build(),
            )
            .then(
                Command::builder()
                    .aur()
                    .args(&["-Rns", "--noconfirm", "python-dlib"])
                    .description("Uninstalling Python dependencies...")
                    .build(),
            )
            .build();

        task_runner::run(
            window_clone.upcast_ref(),
            commands,
            "Uninstall Howdy Qt",
        );
    });
}
