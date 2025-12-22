//! Main page button handlers.
//!
//! Handles:
//! - System update
//! - Package manager GUI installation
//! - Download Arch ISO
//! - External links (Discord, YouTube, Website, Donate)

use crate::config;
use crate::core;
use crate::ui::dialogs::download::show_download_dialog;
use crate::ui::dialogs::selection::{
    show_selection_dialog, SelectionDialogConfig, SelectionOption,
};
use crate::ui::dialogs::terminal;
use crate::ui::task_runner::{self, Command, CommandSequence};
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button};
use log::info;

/// Set up all button handlers for the main page.
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder) {
    setup_update_system(page_builder);
    setup_pkg_manager(page_builder);
    setup_download_arch_iso(page_builder);
    setup_install_nix(page_builder);
    setup_external_links(page_builder);
}

/// Setup system update button.
fn setup_update_system(builder: &Builder) {
    let Some(button) = builder.object::<Button>("btn_update_system") else {
        return;
    };

    button.connect_clicked(move |btn| {
        info!("Update System button clicked");

        let Some(window) = get_window(btn) else {
            return;
        };

        // Use terminal dialog for interactive system update
        terminal::show_terminal_dialog(
            window.upcast_ref(),
            "System Update",
            "/usr/local/bin/upd",
            &[],
        );
    });
}

/// Setup package manager GUI button.
fn setup_pkg_manager(builder: &Builder) {
    let Some(button) = builder.object::<Button>("btn_pkg_manager") else {
        return;
    };

    button.connect_clicked(move |btn| {
        info!("PKG Manager GUI button clicked");

        let Some(window) = get_window(btn) else {
            return;
        };

        let window_clone = window.clone();

        // Check which package managers are already installed
        let config = SelectionDialogConfig::new(
            "Package Manager GUI Applications",
            "Select which package manager GUIs to install. Multiple selections allowed.",
        )
        .add_option(SelectionOption::new(
            "octopi",
            "Octopi",
            "Powerful Pacman GUI with AUR support",
            core::is_package_installed("octopi"),
        ))
        .add_option(SelectionOption::new(
            "pacseek",
            "PacSeek",
            "Terminal UI package manager with search",
            core::is_package_installed("pacseek"),
        ))
        .add_option(SelectionOption::new(
            "bauh",
            "Bauh",
            "Manage Pacman, AUR, Flatpak, Snap packages",
            core::is_package_installed("bauh"),
        ))
        .add_option(SelectionOption::new(
            "warehouse",
            "Warehouse",
            "Flatpak package manager (Flatpak)",
            core::is_flatpak_installed("io.github.flattool.Warehouse"),
        ))
        .add_option(SelectionOption::new(
            "flatseal",
            "Flatseal",
            "Flatpak permissions manager (Flatpak)",
            core::is_flatpak_installed("com.github.tchx84.Flatseal"),
        ))
        .add_option(SelectionOption::new(
            "bazaar",
            "Bazaar",
            "Browse and install Flatpak apps (Flatpak)",
            core::is_flatpak_installed("io.github.kolunmi.Bazaar"),
        ))
        .confirm_label("Install");

        show_selection_dialog(window.upcast_ref(), config, move |selected| {
            let commands = build_pkg_manager_commands(&selected);

            if !commands.is_empty() {
                task_runner::run(
                    window_clone.upcast_ref(),
                    commands.build(),
                    "Package Manager GUI Installation",
                );
            }
        });
    });
}

/// Build commands for selected package managers.
fn build_pkg_manager_commands(selected: &[String]) -> CommandSequence {
    let mut commands = CommandSequence::new();

    if selected.contains(&"octopi".to_string()) {
        commands = commands.then(
            Command::builder()
                .aur()
                .args(&["-S", "--noconfirm", "--needed", "octopi"])
                .description("Installing Octopi package manager...")
                .build(),
        );
    }

    if selected.contains(&"pacseek".to_string()) {
        commands = commands.then(
            Command::builder()
                .aur()
                .args(&["-S", "--noconfirm", "--needed", "pacseek", "pacfinder"])
                .description("Installing PacSeek package browser...")
                .build(),
        );
    }

    if selected.contains(&"bauh".to_string()) {
        commands = commands.then(
            Command::builder()
                .aur()
                .args(&["-S", "--noconfirm", "--needed", "bauh"])
                .description("Installing Bauh package manager...")
                .build(),
        );
    }

    if selected.contains(&"warehouse".to_string()) {
        commands = commands.then(
            Command::builder()
                .normal()
                .program("flatpak")
                .args(&["install", "-y", "io.github.flattool.Warehouse"])
                .description("Installing Warehouse from Flathub...")
                .build(),
        );
    }

    if selected.contains(&"flatseal".to_string()) {
        commands = commands.then(
            Command::builder()
                .normal()
                .program("flatpak")
                .args(&["install", "-y", "com.github.tchx84.Flatseal"])
                .description("Installing Flatseal from Flathub...")
                .build(),
        );
    }

    if selected.contains(&"bazaar".to_string()) {
        commands = commands.then(
            Command::builder()
                .normal()
                .program("flatpak")
                .args(&["install", "-y", "io.github.kolunmi.Bazaar"])
                .description("Installing Bazaar from Flathub...")
                .build(),
        );
    }

    commands
}

/// Setup download Arch ISO button.
fn setup_download_arch_iso(builder: &Builder) {
    let Some(button) = builder.object::<Button>("btn_download_arch_iso") else {
        return;
    };

    button.connect_clicked(move |btn| {
        info!("Download Arch ISO button clicked");

        let Some(window) = get_window(btn) else {
            return;
        };

        show_download_dialog(window.upcast_ref());
    });
}

/// Setup Nix package manager installation button.
fn setup_install_nix(builder: &Builder) {
    let Some(button) = builder.object::<Button>("btn_install_nix") else {
        return;
    };

    button.connect_clicked(move |btn| {
        info!("Install Nix button clicked");

        let Some(window) = get_window(btn) else {
            return;
        };

        let window_clone = window.clone();

        // Build Nix installation command sequence
        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .privileged()
                    .program("pacman")
                    .args(&["-Sy", "--noconfirm", "curl", "xz"])
                    .description("Installing basic dependencies (curl, xz)...")
                    .build(),
            )
            .then(
                Command::builder()
                    .normal()
                    .program("bash")
                    .args(&[
                        "-c",
                        "rm -rf ~/.nix-profile ~/.nix-defexpr ~/.nix-channels ~/.config/nix",
                    ])
                    .description("Removing previous Nix installation remnants...")
                    .build(),
            )
            .then(
                Command::builder()
                    .normal()
                    .program("curl")
                    .args(&["-L", "https://nixos.org/nix/install", "-o", "/tmp/nix-install.sh"])
                    .description("Downloading Nix installer...")
                    .build(),
            )
            .then(
                Command::builder()
                    .normal()
                    .program("bash")
                    .args(&["/tmp/nix-install.sh", "--daemon"])
                    .description("Running Nix installer in daemon mode...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["daemon-reexec"])
                    .description("Reloading systemd daemon...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["daemon-reload"])
                    .description("Reloading systemd configuration...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["enable", "--now", "nix-daemon.service"])
                    .description("Enabling and starting nix-daemon service...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("mkdir")
                    .args(&["-p", "/etc/nix"])
                    .description("Creating /etc/nix directory...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("bash")
                    .args(&[
                        "-c",
                        "echo 'experimental-features = nix-command' | tee /etc/nix/nix.conf",
                    ])
                    .description("Configuring Nix experimental features...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("bash")
                    .args(&[
                        "-c",
                        "if ! grep -qF '. /etc/profile.d/nix.sh' /etc/profile; then echo '. /etc/profile.d/nix.sh' | tee -a /etc/profile; fi",
                    ])
                    .description("Configuring Bash shell integration...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("bash")
                    .args(&[
                        "-c",
                        "if command -v zsh >/dev/null 2>&1 && [ -f /etc/zsh/zprofile ] && ! grep -qF '. /etc/profile.d/nix.sh' /etc/zsh/zprofile; then echo '. /etc/profile.d/nix.sh' | tee -a /etc/zsh/zprofile; fi",
                    ])
                    .description("Configuring Zsh shell integration...")
                    .build(),
            )
            .then(
                Command::builder()
                    .normal()
                    .program("bash")
                    .args(&[
                        "-c",
                        r##"if [ -f ~/.bashrc ]; then { echo ""; echo "# Nix aliases"; echo "nstall() { nix-env -iA \"nixpkgs.\$1\"; }"; echo "nsearch() { nix-env -qaP \"\$@\" 2>&1 | grep -vE '^(evaluation warning:|warning: name collision)'; }"; } >> ~/.bashrc; fi"##,
                    ])
                    .description("Adding Nix aliases to .bashrc...")
                    .build(),
            )
            .then(
                Command::builder()
                    .normal()
                    .program("bash")
                    .args(&[
                        "-c",
                        r##"if [ -f ~/.zshrc ]; then { echo ""; echo "# Nix aliases"; echo "nstall() { nix-env -iA \"nixpkgs.\$1\"; }"; echo "nsearch() { nix-env -qaP \"\$@\" 2>&1 | grep -vE '^(evaluation warning:|warning: name collision)'; }"; } >> ~/.zshrc; fi"##,
                    ])
                    .description("Adding Nix aliases to .zshrc...")
                    .build(),
            )
            .build();

        task_runner::run(
            window_clone.upcast_ref(),
            commands,
            "Install Nix Package Manager",
        );
    });
}

/// Setup external link buttons.
fn setup_external_links(builder: &Builder) {
    if let Some(btn) = builder.object::<Button>("link_discord") {
        btn.connect_clicked(|_| {
            info!("Discord link clicked");
            let _ = core::package::open_url(config::links::DISCORD);
        });
    }

    if let Some(btn) = builder.object::<Button>("link_youtube") {
        btn.connect_clicked(|_| {
            info!("YouTube link clicked");
            let _ = core::package::open_url(config::links::YOUTUBE);
        });
    }

    if let Some(btn) = builder.object::<Button>("link_website") {
        btn.connect_clicked(|_| {
            info!("Website link clicked");
            let _ = core::package::open_url(config::links::WEBSITE);
        });
    }

    if let Some(btn) = builder.object::<Button>("link_donate") {
        btn.connect_clicked(|_| {
            info!("Donate link clicked");
            let _ = core::package::open_url(config::links::DONATE);
        });
    }
}

/// Helper to get the parent window from a button.
fn get_window(button: &Button) -> Option<ApplicationWindow> {
    button
        .root()
        .and_then(|root| root.downcast::<ApplicationWindow>().ok())
}
