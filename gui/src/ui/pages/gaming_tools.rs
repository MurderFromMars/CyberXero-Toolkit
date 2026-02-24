//! Gaming tools page button handlers.
//!
//! Handles:
//! - Gaming suite installation (CachyOS meta or curated package list fallback)
//! - LACT GPU overclocking
//! - Game launchers (Bottles)
//! - Controller tools
//! - Falcond gaming utility

use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::extract_widget;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button};
use log::info;

/// Set up all button handlers for the gaming tools page.
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    setup_gaming_meta(page_builder, window);
    setup_lact_oc(page_builder, window);
    setup_bottles(page_builder, window);
    setup_controller(page_builder, window);
    setup_falcond(page_builder, window);
}

fn setup_gaming_meta(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_gaming_meta");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Gaming Suite button clicked");

        let mut commands = CommandSequence::new();

        // Check if CachyOS gaming packages are available in repos
        let cachy_meta_available = crate::core::is_package_in_repos("cachyos-gaming-meta");
        let cachy_apps_available = crate::core::is_package_in_repos("cachyos-gaming-applications");

        if cachy_meta_available && cachy_apps_available {
            info!("CachyOS gaming packages found in repos, installing from repos");
            commands = commands.then(
                Command::builder()
                    .privileged()
                    .program("pacman")
                    .args(&[
                        "-S",
                        "--noconfirm",
                        "--needed",
                        "cachyos-gaming-meta",
                        "cachyos-gaming-applications",
                    ])
                    .description("Installing CachyOS gaming meta packages...")
                    .build(),
            );
        } else {
            info!("CachyOS gaming packages not in repos, installing curated gaming suite");

            // -- Step 1: Multilib runtime libraries & Wine/Proton stack --
            commands = commands.then(
                Command::builder()
                    .privileged()
                    .program("pacman")
                    .args(&[
                        "-S",
                        "--noconfirm",
                        "--needed",
                        // Audio
                        "alsa-plugins",
                        "lib32-alsa-plugins",
                        // Media/codec libs
                        "giflib",
                        "lib32-giflib",
                        "gst-plugins-base-libs",
                        "lib32-gst-plugins-base-libs",
                        "libjpeg-turbo",
                        "lib32-libjpeg-turbo",
                        "mpg123",
                        "lib32-mpg123",
                        "libxslt",
                        "openal",
                        "lib32-openal",
                        // Video acceleration
                        "libva",
                        "lib32-libva",
                        // OpenCL
                        "opencl-icd-loader",
                        "lib32-opencl-icd-loader",
                        // Vulkan
                        "vulkan-icd-loader",
                        "lib32-vulkan-icd-loader",
                        "vulkan-tools",
                        // GTK (needed by some launchers/games)
                        "lib32-gtk3",
                        // GLFW
                        "glfw",
                        // Fonts (required by many Windows games)
                        "ttf-liberation",
                        "wqy-zenhei",
                        // Wine & Proton tools
                        "wine-staging",
                        "winetricks",
                        "protontricks",
                        "umu-launcher",
                        // Gaming tools & launchers
                        "steam",
                        "gamescope",
                        "gamemode",
                        "lib32-gamemode",
                        "mangohud",
                        "lib32-mangohud",
                        "goverlay",
                        "lutris",
                    ])
                    .description("Installing gaming libraries, Wine, and tools from repos...")
                    .build(),
            );

            // -- Step 2: AUR packages --
            commands = commands.then(
                Command::builder()
                    .aur()
                    .args(&[
                        "-S",
                        "--noconfirm",
                        "--needed",
                        "heroic-games-launcher-bin",
                    ])
                    .description("Installing Heroic Games Launcher from AUR...")
                    .build(),
            );

            // -- Step 3: Splitlock mitigation disable (gaming perf optimization) --
            commands = commands.then(
                Command::builder()
                    .privileged()
                    .program("sh")
                    .args(&[
                        "-c",
                        "echo 'kernel.split_lock_mitigate=0' > /etc/sysctl.d/99-splitlock.conf && sysctl --system",
                    ])
                    .description("Disabling split-lock mitigation for gaming performance...")
                    .build(),
            );
        }

        task_runner::run(window.upcast_ref(), commands.build(), "Gaming Suite Installation");
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
