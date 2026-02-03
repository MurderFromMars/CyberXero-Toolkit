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
fn update_button_state(button: &gtk4::Button, is_installed: bool) {
    if is_installed {
        button.set_label("Launch App");
        button.add_css_class("suggested-action");
    } else {
        button.set_label("Install");
        button.remove_css_class("suggested-action");
    }
}

fn setup_fingerprint(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_fingerprint_setup =
        extract_widget::<gtk4::Button>(page_builder, "btn_fingerprint_setup");

    // Initial check
    let is_installed = core::is_package_installed("xfprintd-gui");
    update_button_state(&btn_fingerprint_setup, is_installed);

    // Update on window focus (e.g. after installation completes)
    let btn_clone = btn_fingerprint_setup.clone();
    window.connect_is_active_notify(move |window| {
        if window.is_active() {
            let is_installed = core::is_package_installed("xfprintd-gui");
            update_button_state(&btn_clone, is_installed);
        }
    });

    let window = window.clone();
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
                window.upcast_ref(),
                commands,
                "Install Fingerprint GUI Tool",
            );
        }
    });
}

fn setup_howdy(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_howdy_setup = extract_widget::<gtk4::Button>(page_builder, "btn_howdy_setup");

    // Initial check
    let is_installed = core::is_package_installed("xero-howdy-qt");
    update_button_state(&btn_howdy_setup, is_installed);

    // Update on window focus (e.g. after installation completes)
    let btn_clone = btn_howdy_setup.clone();
    window.connect_is_active_notify(move |window| {
        if window.is_active() {
            let is_installed = core::is_package_installed("xero-howdy-qt");
            update_button_state(&btn_clone, is_installed);
        }
    });

    let window = window.clone();
    btn_howdy_setup.connect_clicked(move |_| {
        info!("Biometrics: Howdy setup button clicked");

        // Check again at click time
        if core::is_package_installed("xero-howdy-qt") {
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
            let commands = CommandSequence::new()
                .then(
                    Command::builder()
                        .aur()
                        .args(&["-S", "--noconfirm", "--needed", "rust", "cargo", "clang", "qt6-base", "qt6-declarative", "howdy-git"])
                        .description("Installing build dependencies...")
                        .build(),
                )
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

            task_runner::run(window.upcast_ref(), commands, "Install Howdy Qt (Build from Source)");
        }
    });
}
