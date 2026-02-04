//! Gaming tools page button handlers.
//!
//! Handles:
//! - Steam AiO installation
//! - LACT GPU overclocking
//! - Game launchers (Lutris, Heroic, Bottles)
//! - Controller tools
//! - Falcond gaming utility

use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::extract_widget;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button};
use log::info;

/// Set up all button handlers for the gaming tools page.
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    setup_steam_aio(page_builder, window);
    setup_lact_oc(page_builder, window);
    setup_lutris(page_builder, window);
    setup_heroic(page_builder, window);
    setup_bottles(page_builder, window);
    setup_controller(page_builder, window);
    setup_falcond(page_builder, window);
}

fn setup_steam_aio(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_steam_aio");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Steam AiO button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&[
                        "-S",
                        "--noconfirm",
                        "--needed",
                        "steam",
                        "gamescope",
                        "mangohud",
                        "mangoverlay",
                        "lib32-mangohud",
                        "protonplus",
                        "wine-meta",
                        "wine-nine",
                        "ttf-liberation",
                        "lib32-fontconfig",
                        "wqy-zenhei",
                        "vkd3d",
                        "giflib",
                        "lib32-giflib",
                        "libpng",
                        "lib32-libpng",
                        "libldap",
                        "lib32-libldap",
                        "gnutls",
                        "lib32-gnutls",
                        "mpg123",
                        "lib32-mpg123",
                        "openal",
                        "lib32-openal",
                        "v4l-utils",
                        "lib32-v4l-utils",
                        "libpulse",
                        "lib32-libpulse",
                        "libgpg-error",
                        "lib32-libgpg-error",
                        "alsa-plugins",
                        "lib32-alsa-plugins",
                        "alsa-lib",
                        "lib32-alsa-lib",
                        "libjpeg-turbo",
                        "lib32-libjpeg-turbo",
                        "sqlite",
                        "lib32-sqlite",
                        "libxcomposite",
                        "lib32-libxcomposite",
                        "libxinerama",
                        "lib32-libgcrypt",
                        "libgcrypt",
                        "lib32-libxinerama",
                        "ncurses",
                        "lib32-ncurses",
                        "ocl-icd",
                        "lib32-ocl-icd",
                        "libxslt",
                        "lib32-libxslt",
                        "libva",
                        "lib32-libva",
                        "gtk3",
                        "lib32-gtk3",
                        "gst-plugins-base-libs",
                        "lib32-gst-plugins-base-libs",
                        "vulkan-icd-loader",
                        "lib32-vulkan-icd-loader",
                        "cups",
                        "dosbox",
                        "lib32-opencl-icd-loader",
                        "lib32-vkd3d",
                        "opencl-icd-loader",
                    ])
                    .description("Installing Steam and gaming dependencies...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "Steam AiO Installation");
    });
}

fn setup_lact_oc(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_lact_oc");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("LACT OC button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&["-S", "--noconfirm", "--needed", "lact"])
                    .description("Installing LACT GPU control utility...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["enable", "--now", "lactd"])
                    .description("Enabling LACT background service...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "LACT GPU Tools");
    });
}

fn setup_lutris(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_lutris");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Lutris button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .normal()
                    .program("flatpak")
                    .args(&[
                        "install",
                        "-y",
                        "net.lutris.Lutris",
                        "org.freedesktop.Platform.VulkanLayer.gamescope/x86_64/25.08",
                        "org.freedesktop.Platform.VulkanLayer.MangoHud/x86_64/25.08",
                    ])
                    .description("Installing Lutris and Vulkan layers...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "Lutris Installation");
    });
}

fn setup_heroic(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_heroic");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Heroic button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .normal()
                    .program("flatpak")
                    .args(&[
                        "install",
                        "-y",
                        "com.heroicgameslauncher.hgl",
                        "org.freedesktop.Platform.VulkanLayer.gamescope/x86_64/25.08",
                        "org.freedesktop.Platform.VulkanLayer.MangoHud/x86_64/25.08",
                    ])
                    .description("Installing Heroic Games Launcher...")
                    .build(),
            )
            .build();

        task_runner::run(
            window.upcast_ref(),
            commands,
            "Heroic Launcher Installation",
        );
    });
}

fn setup_bottles(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_bottles");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Bottles button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .normal()
                    .program("flatpak")
                    .args(&[
                        "install",
                        "-y",
                        "com.usebottles.bottles",
                        "org.freedesktop.Platform.VulkanLayer.gamescope/x86_64/25.08",
                        "org.freedesktop.Platform.VulkanLayer.MangoHud/x86_64/25.08",
                    ])
                    .description("Installing Bottles and Vulkan layers...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "Bottles Installation");
    });
}

fn setup_controller(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_controller");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Controller Tools button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&[
                        "-S",
                        "--noconfirm",
                        "--needed",
                        "gamepad-tool-bin",
                        "sc-controller",
                        "xone-dkms-git",
                        "dualsensectl-git",
                        "xone-dongle-firmware",
                    ])
                    .description("Installing controller tools and drivers...")
                    .build(),
            )
            .build();

        task_runner::run(
            window.upcast_ref(),
            commands,
            "Controller Tools Installation",
        );
    });
}

fn setup_falcond(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_falcond");
    let window = window.clone();

    let env = crate::config::env::get();
    let user = env.user.clone();

    button.connect_clicked(move |_| {
        info!("Falcond button clicked");

        let mut commands = CommandSequence::new();
        
        // Remove power-profiles-daemon if installed (conflicts with tuned-ppd)
        if crate::core::is_package_installed("power-profiles-daemon") {
            info!("power-profiles-daemon installed, removing first (conflicts with tuned-ppd)");
            commands = commands.then(
                Command::builder()
                    .privileged()
                    .program("sh")
                    .args(&["-c", "pacman -Rns --noconfirm power-profiles-daemon || true"])
                    .description("Removing power-profiles-daemon (conflicts with tuned-ppd)...")
                    .build(),
            );
        }
        
        // Packages to install
        let repo_candidates = ["falcond", "falcond-gui", "tuned-ppd"];
        
        let mut pacman_packages: Vec<&str> = Vec::new();
        let mut aur_packages: Vec<&str> = Vec::new();
        let mut all_in_repos = true;
        
        for pkg in repo_candidates {
            // Skip if already installed
            if crate::core::is_package_installed(pkg) {
                info!("{} already installed, skipping", pkg);
                continue;
            }
            
            // Check if available in repos
            if crate::core::is_package_in_repos(pkg) {
                info!("{} found in repos", pkg);
                pacman_packages.push(pkg);
            } else {
                info!("{} not in repos, will use AUR", pkg);
                aur_packages.push(pkg);
                all_in_repos = false;
            }
        }
        
        // If any package needs AUR, add falcond-profiles too (AUR-only)
        if !all_in_repos && !crate::core::is_package_installed("falcond-profiles") {
            info!("falcond-profiles not installed, adding to AUR list");
            aur_packages.push("falcond-profiles");
        }
        
        // Install from repos first
        if !pacman_packages.is_empty() {
            let mut args = vec!["-S", "--noconfirm", "--needed"];
            args.extend(pacman_packages.iter());
            
            commands = commands.then(
                Command::builder()
                    .privileged()
                    .program("pacman")
                    .args(&args)
                    .description("Installing Falcond packages from repos...")
                    .build(),
            );
        }
        
        // Install remaining from AUR (only if needed)
        if !aur_packages.is_empty() {
            let mut args = vec!["-S", "--noconfirm", "--needed"];
            args.extend(aur_packages.iter());
            
            commands = commands.then(
                Command::builder()
                    .aur()
                    .args(&args)
                    .description("Installing Falcond packages from AUR...")
                    .build(),
            );
        }
        
        // Post-install setup (always run to ensure proper configuration)
        commands = commands
            .then(
                Command::builder()
                    .privileged()
                    .program("groupadd")
                    .args(&["-f", "falcond"])
                    .description("Ensuring falcond group exists...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("usermod")
                    .args(&["-aG", "falcond", &user])
                    .description("Adding your user to falcond group...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("mkdir")
                    .args(&["-p", "/usr/share/falcond/profiles/user"])
                    .description("Creating necessary user directory...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("chown")
                    .args(&[":falcond", "/usr/share/falcond/profiles/user"])
                    .description("Adding proper ownership permissions...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("chmod")
                    .args(&["2775", "/usr/share/falcond/profiles/user"])
                    .description("Adding proper executable permissions...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("systemctl")
                    .args(&["enable", "--now", "falcond"])
                    .description("Enabling falcond background service...")
                    .build(),
            );

        task_runner::run(window.upcast_ref(), commands.build(), "Falcond Installation");
    });
}
