//! Drivers and hardware tools page button handlers.
//!
//! Handles:
//! - Tailscale VPN
//! - ASUS ROG laptop tools
//! - OpenRazer drivers

use crate::core;
use crate::ui::dialogs::selection::{
    show_selection_dialog, SelectionDialogConfig, SelectionOption,
};
use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::extract_widget;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button};
use log::info;

/// Set up all button handlers for the drivers page.
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    setup_tailscale(page_builder, window);
    setup_asus_rog(page_builder, window);
    setup_openrazer(page_builder, window);
}

fn setup_tailscale(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_tailscale");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Tailscale VPN button clicked");

        let commands = CommandSequence::new()
            .then(Command::builder()
                .privileged()
                .program("bash")
                .args(&[
                    "-c",
                    "curl -fsSL https://raw.githubusercontent.com/xerolinux/xero-fixes/main/conf/install.sh | bash",
                ])
                .description("Installing Tailscale VPN...")
                .build())
            .build();

        task_runner::run(window.upcast_ref(), commands, "Install Tailscale VPN");
    });
}

fn setup_asus_rog(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_asus_rog");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("ASUS ROG Tools button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&[
                        "-S",
                        "--noconfirm",
                        "--needed",
                        "rog-control-center",
                        "asusctl",
                        "supergfxctl",
                    ])
                    .description("Installing ASUS ROG control tools...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["enable", "--now", "asusd", "supergfxd"])
                    .description("Enabling ASUS ROG services...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "Install ASUS ROG Tools");
    });
}

fn setup_openrazer(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_openrazer");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("OpenRazer Drivers button clicked");

        // Show selection dialog for optional frontends
        let window_clone = window.clone();
        let config = SelectionDialogConfig::new(
            "OpenRazer Drivers & Frontend",
            "OpenRazer drivers will be installed. Optionally select a frontend application for managing your Razer devices.",
        )
        .add_option(SelectionOption::new(
            "polychromatic",
            "Polychromatic",
            "Graphical frontend for managing Razer devices (GTK-based)",
            core::is_package_installed("polychromatic"),
        ))
        .add_option(SelectionOption::new(
            "razergenie",
            "RazerGenie",
            "Graphical frontend for managing Razer devices (Qt-based)",
            core::is_package_installed("razergenie"),
        ))
        .confirm_label("Install");

        show_selection_dialog(window.upcast_ref(), config, move |selected| {
            let commands = build_openrazer_commands(&selected);
            task_runner::run(
                window_clone.upcast_ref(),
                commands,
                "Install OpenRazer Drivers (Reboot Required)",
            );
        });
    });
}

/// Build commands for OpenRazer installation.
fn build_openrazer_commands(selected_frontends: &[String]) -> CommandSequence {
    let user = crate::config::env::get().user.clone();
    let mut commands = CommandSequence::new();

    // Always install openrazer-meta-git
    commands = commands.then(
        Command::builder()
            .aur()
            .args(&["-S", "--noconfirm", "--needed", "openrazer-meta-git"])
            .description("Installing OpenRazer drivers...")
            .build(),
    );

    // Add user to plugdev group
    commands = commands.then(
        Command::builder()
            .privileged()
            .program("usermod")
            .args(&["-aG", "plugdev", &user])
            .description("Adding user to plugdev group...")
            .build(),
    );

    // Optionally install selected frontends
    if selected_frontends.contains(&"polychromatic".to_string()) {
        commands = commands.then(
            Command::builder()
                .aur()
                .args(&["-S", "--noconfirm", "--needed", "polychromatic"])
                .description("Installing Polychromatic frontend...")
                .build(),
        );
    }

    if selected_frontends.contains(&"razergenie".to_string()) {
        commands = commands.then(
            Command::builder()
                .aur()
                .args(&["-S", "--noconfirm", "--needed", "razergenie"])
                .description("Installing RazerGenie frontend...")
                .build(),
        );
    }

    commands
}
