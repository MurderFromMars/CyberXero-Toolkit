//! Main page button handlers.
//!
//! Handles:
//! - System update
//! - Package manager GUI installation
//! - Driver installation (GPU, Tailscale, ASUS ROG)
//! - Parallel downloads setup
//! - External links (Discord, YouTube, Website, Donate)

use crate::core;
use crate::ui::selection_dialog;
use crate::ui::terminal;
use crate::utils;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button, Label};
use log::{info, warn};
use vte4::prelude::*;
use vte4::Terminal;

/// Set up all button handlers for the main page
pub fn setup_handlers(page_builder: &Builder, main_builder: &Builder) {
    setup_update_system_button(page_builder, main_builder);
    setup_pkg_manager_button(page_builder, main_builder);
    setup_parallel_downloads_button(page_builder, main_builder);
    setup_install_drivers_button(page_builder, main_builder);
    setup_external_links(page_builder);
}

/// Setup system update button
fn setup_update_system_button(page_builder: &Builder, main_builder: &Builder) {
    let terminal_output: Terminal = main_builder
        .object("global_terminal_output_view")
        .expect("Failed to get terminal output view");
    let terminal_title: Label = main_builder
        .object("global_terminal_title")
        .expect("Failed to get terminal title label");

    if let Some(btn_update_system) = page_builder.object::<Button>("btn_update_system") {
        let terminal_clone = terminal_output.clone();
        let title_clone = terminal_title.clone();
        let button_clone = btn_update_system.clone();

        btn_update_system.connect_clicked(move |_| {
            info!("Main page: Update System button clicked");

            if terminal::is_action_running() {
                warn!("Action already running, ignoring button click");
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
                terminal::TerminalCommand::new(helper, &["-Syu"]),
            ];

            terminal::run_terminal_commands(
                &button_clone,
                &terminal_clone,
                &title_clone,
                commands,
                "System Update",
            );
        });
    }
}

/// Setup package manager GUI button
fn setup_pkg_manager_button(page_builder: &Builder, main_builder: &Builder) {
    let terminal_output: Terminal = main_builder
        .object("global_terminal_output_view")
        .expect("Failed to get terminal output view");
    let terminal_title: Label = main_builder
        .object("global_terminal_title")
        .expect("Failed to get terminal title label");

    if let Some(btn_pkg_manager) = page_builder.object::<Button>("btn_pkg_manager") {
        let terminal_clone = terminal_output.clone();
        let title_clone = terminal_title.clone();

        btn_pkg_manager.connect_clicked(move |button| {
            info!("Main page: PKG Manager GUI button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(b"\r\nAnother action is already running. Please wait for it to complete.\r\n");
                return;
            }

            show_pkg_manager_dialog(button, &terminal_clone, &title_clone);
        });
    }
}

/// Setup parallel downloads button
fn setup_parallel_downloads_button(page_builder: &Builder, main_builder: &Builder) {
    let terminal_output: Terminal = main_builder
        .object("global_terminal_output_view")
        .expect("Failed to get terminal output view");
    let terminal_title: Label = main_builder
        .object("global_terminal_title")
        .expect("Failed to get terminal title label");

    if let Some(btn_parallel_downloads) = page_builder.object::<Button>("btn_parallel_downloads") {
        let terminal_clone = terminal_output.clone();
        let title_clone = terminal_title.clone();
        let button_clone = btn_parallel_downloads.clone();

        btn_parallel_downloads.connect_clicked(move |_| {
            info!("Main page: Setup Parallel Downloads button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(b"\r\nAnother action is already running. Please wait for it to complete.\r\n");
                return;
            }

            // Run pmpd to configure parallel downloads
            let commands = vec![
                terminal::TerminalCommand::new("sudo", &["pmpd"]),
            ];

            terminal::run_terminal_commands(
                &button_clone,
                &terminal_clone,
                &title_clone,
                commands,
                "Setup Parallel Downloads",
            );
        });
    }
}

/// Setup install drivers button
fn setup_install_drivers_button(page_builder: &Builder, main_builder: &Builder) {
    let terminal_output: Terminal = main_builder
        .object("global_terminal_output_view")
        .expect("Failed to get terminal output view");
    let terminal_title: Label = main_builder
        .object("global_terminal_title")
        .expect("Failed to get terminal title label");

    if let Some(btn_install_drivers) = page_builder.object::<Button>("btn_install_drivers") {
        let terminal_clone = terminal_output.clone();
        let title_clone = terminal_title.clone();

        btn_install_drivers.connect_clicked(move |button| {
            info!("Main page: Install Drivers button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(b"\r\nAnother action is already running. Please wait for it to complete.\r\n");
                return;
            }

            show_driver_installation_dialog(button, &terminal_clone, &title_clone);
        });
    }
}

/// Setup external link buttons
fn setup_external_links(page_builder: &Builder) {
    if let Some(link_discord) = page_builder.object::<Button>("link_discord") {
        link_discord.connect_clicked(move |_| {
            info!("Main page: Discord link clicked");
            let _ = std::process::Command::new("xdg-open")
                .arg("https://discord.xerolinux.xyz/")
                .spawn();
        });
    }

    if let Some(link_youtube) = page_builder.object::<Button>("link_youtube") {
        link_youtube.connect_clicked(move |_| {
            info!("Main page: YouTube link clicked");
            let _ = std::process::Command::new("xdg-open")
                .arg("https://www.youtube.com/@XeroLinux")
                .spawn();
        });
    }

    if let Some(link_website) = page_builder.object::<Button>("link_website") {
        link_website.connect_clicked(move |_| {
            info!("Main page: XeroLinux website link clicked");
            let _ = std::process::Command::new("xdg-open")
                .arg("https://xerolinux.xyz/")
                .spawn();
        });
    }

    if let Some(link_donate) = page_builder.object::<Button>("link_donate") {
        link_donate.connect_clicked(move |_| {
            info!("Main page: Donate link clicked");
            let _ = std::process::Command::new("xdg-open")
                .arg("https://ko-fi.com/xerolinux")
                .spawn();
        });
    }
}

/// Show package manager selection dialog
fn show_pkg_manager_dialog(button: &Button, terminal: &Terminal, terminal_title: &Label) {
    let widget = button.clone().upcast::<gtk4::Widget>();
    let window = widget.root()
        .and_then(|root| root.downcast::<ApplicationWindow>().ok());

    if let Some(window) = window {
        let terminal_for_dialog = terminal.clone();
        let title_for_dialog = terminal_title.clone();
        let button_for_dialog = button.clone();
        let window_ref = window.upcast_ref::<gtk4::Window>();

        // Check which package managers are already installed
        let octopi_installed = core::is_package_installed("octopi");
        let pacseek_installed = core::is_package_installed("pacseek");
        let bauh_installed = core::is_package_installed("bauh");
        let warehouse_installed = core::is_flatpak_installed("io.github.flattool.Warehouse");
        let flatseal_installed = core::is_flatpak_installed("com.github.tchx84.Flatseal");
        let bazaar_installed = core::is_flatpak_installed("io.github.kolunmi.Bazaar");

        let config = selection_dialog::SelectionDialogConfig::new(
            "Package Manager GUI Applications",
            "Select which package manager GUIs to install. Multiple selections allowed.",
        )
        .add_option(selection_dialog::SelectionOption::new(
            "octopi",
            "Octopi",
            "Powerful Pacman GUI with AUR support",
            octopi_installed,
        ))
        .add_option(selection_dialog::SelectionOption::new(
            "pacseek",
            "PacSeek",
            "Terminal UI package manager with search",
            pacseek_installed,
        ))
        .add_option(selection_dialog::SelectionOption::new(
            "bauh",
            "Bauh",
            "Manage Pacman, AUR, Flatpak, Snap packages",
            bauh_installed,
        ))
        .add_option(selection_dialog::SelectionOption::new(
            "warehouse",
            "Warehouse",
            "Flatpak package manager (Flatpak)",
            warehouse_installed,
        ))
        .add_option(selection_dialog::SelectionOption::new(
            "flatseal",
            "Flatseal",
            "Flatpak permissions manager (Flatpak)",
            flatseal_installed,
        ))
        .add_option(selection_dialog::SelectionOption::new(
            "bazaar",
            "Bazaar",
            "Browse and install Flatpak apps (Flatpak)",
            bazaar_installed,
        ))
        .confirm_label("Install");

        selection_dialog::show_selection_dialog(
            window_ref,
            config,
            move |selected_ids| {
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

                let mut commands = vec![];

                if selected_ids.contains(&"octopi".to_string()) {
                    commands.push(terminal::TerminalCommand::new(helper,
                        &["-S", "--noconfirm", "--needed", "octopi"]));
                }

                if selected_ids.contains(&"pacseek".to_string()) {
                    commands.push(terminal::TerminalCommand::new(helper,
                        &["-S", "--noconfirm", "--needed", "pacseek", "pacfinder"]));
                }

                if selected_ids.contains(&"bauh".to_string()) {
                    commands.push(terminal::TerminalCommand::new(helper,
                        &["-S", "--noconfirm", "--needed", "bauh"]));
                }

                if selected_ids.contains(&"warehouse".to_string()) {
                    commands.push(terminal::TerminalCommand::new("flatpak",
                        &["install", "-y", "io.github.flattool.Warehouse"]));
                }

                if selected_ids.contains(&"flatseal".to_string()) {
                    commands.push(terminal::TerminalCommand::new("flatpak",
                        &["install", "-y", "com.github.tchx84.Flatseal"]));
                }

                if selected_ids.contains(&"bazaar".to_string()) {
                    commands.push(terminal::TerminalCommand::new("flatpak",
                        &["install", "-y", "io.github.kolunmi.Bazaar"]));
                }

                if !commands.is_empty() {
                    terminal::run_terminal_commands(
                        &button_for_dialog,
                        &terminal_for_dialog,
                        &title_for_dialog,
                        commands,
                        "Package Manager GUI Installation",
                    );
                }
            },
        );
    }
}

/// Show driver installation dialog
fn show_driver_installation_dialog(button: &Button, terminal: &Terminal, terminal_title: &Label) {
    let widget = button.clone().upcast::<gtk4::Widget>();
    let window = widget.root()
        .and_then(|root| root.downcast::<ApplicationWindow>().ok());

    if let Some(window) = window {
        let terminal_for_dialog = terminal.clone();
        let title_for_dialog = terminal_title.clone();
        let button_for_dialog = button.clone();
        let window_ref = window.upcast_ref::<gtk4::Window>();

        let config = selection_dialog::SelectionDialogConfig::new(
            "Device Drivers Installation",
            "Select which drivers/tools to install. GPU selection will show a second dialog for vendor choice.",
        )
        .add_option(selection_dialog::SelectionOption::new(
            "gpu",
            "GPU Drivers & Codecs",
            "Intel/AMD/NVIDIA drivers with configuration",
            false,
        ))
        .add_option(selection_dialog::SelectionOption::new(
            "tailscale",
            "Tailscale VPN",
            "Tailscale with XeroLinux configuration",
            false,
        ))
        .add_option(selection_dialog::SelectionOption::new(
            "asus_rog",
            "ASUS ROG Laptop Tools",
            "ROG Control Center, asusctl, supergfxctl",
            false,
        ))
        .confirm_label("Install");

        selection_dialog::show_selection_dialog(
            window_ref,
            config,
            move |selected_ids| {
                if selected_ids.is_empty() {
                    return;
                }

                // Handle GPU drivers (interactive)
                if selected_ids.contains(&"gpu".to_string()) {
                    show_gpu_driver_selection(&button_for_dialog, &terminal_for_dialog, &title_for_dialog);
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

                let mut commands = vec![];

                if selected_ids.contains(&"tailscale".to_string()) {
                    commands.push(terminal::TerminalCommand::new("bash",
                        &["-c", "curl -fsSL https://raw.githubusercontent.com/xerolinux/xero-fixes/main/conf/install.sh | bash"]));
                }

                if selected_ids.contains(&"asus_rog".to_string()) {
                    commands.push(terminal::TerminalCommand::new(helper,
                        &["-S", "--noconfirm", "--needed", "rog-control-center", "asusctl", "supergfxctl"]));
                    commands.push(terminal::TerminalCommand::new("sudo",
                        &["systemctl", "enable", "--now", "asusd", "supergfxd"]));
                }

                if !commands.is_empty() {
                    terminal::run_terminal_commands(
                        &button_for_dialog,
                        &terminal_for_dialog,
                        &title_for_dialog,
                        commands,
                        "Driver/Tools Installation",
                    );
                }
            },
        );
    }
}

/// Show GPU driver vendor selection dialog
fn show_gpu_driver_selection(button: &Button, terminal: &Terminal, terminal_title: &Label) {
    let widget = button.clone().upcast::<gtk4::Widget>();
    let window = widget.root()
        .and_then(|root| root.downcast::<ApplicationWindow>().ok());

    if let Some(window) = window {
        let terminal_for_dialog = terminal.clone();
        let title_for_dialog = terminal_title.clone();
        let button_for_dialog = button.clone();
        let window_ref = window.upcast_ref::<gtk4::Window>();

        let config = selection_dialog::SelectionDialogConfig::new(
            "GPU Driver Selection",
            "Select your GPU vendor. Note: Dual/Hybrid GPU setups require manual terminal configuration.",
        )
        .add_option(selection_dialog::SelectionOption::new(
            "amd",
            "AMD GPU",
            "Radeon graphics drivers and Vulkan support",
            false,
        ))
        .add_option(selection_dialog::SelectionOption::new(
            "intel",
            "Intel GPU",
            "Intel graphics drivers and media acceleration",
            false,
        ))
        .add_option(selection_dialog::SelectionOption::new(
            "nvidia_closed",
            "NVIDIA (Closed Source)",
            "Proprietary NVIDIA drivers with CUDA support",
            false,
        ))
        .add_option(selection_dialog::SelectionOption::new(
            "nvidia_open",
            "NVIDIA (Open Source)",
            "Open source NVIDIA drivers (Turing+ GPUs)",
            false,
        ))
        .confirm_label("Install");

        selection_dialog::show_selection_dialog(
            window_ref,
            config,
            move |selected_ids| {
                if selected_ids.is_empty() {
                    return;
                }

                let mut commands = vec![];

                if selected_ids.contains(&"amd".to_string()) {
                    commands.push(terminal::TerminalCommand::new("sudo",
                        &["pacman", "-S", "--needed", "--noconfirm",
                          "linux-headers", "vulkan-radeon", "lib32-vulkan-radeon",
                          "vulkan-icd-loader", "lib32-vulkan-icd-loader",
                          "linux-firmware-radeon",
                          "vulkan-mesa-layers", "lib32-vulkan-mesa-layers"]));
                }

                if selected_ids.contains(&"intel".to_string()) {
                    commands.push(terminal::TerminalCommand::new("sudo",
                        &["pacman", "-S", "--needed", "--noconfirm",
                          "linux-headers", "vulkan-intel", "lib32-vulkan-intel",
                          "vulkan-icd-loader", "lib32-vulkan-icd-loader",
                          "intel-media-driver", "intel-gmmlib", "onevpl-intel-gpu",
                          "gstreamer-vaapi"]));
                }

                if selected_ids.contains(&"nvidia_closed".to_string()) {
                    commands.push(terminal::TerminalCommand::new("sudo",
                        &["pacman", "-S", "--needed", "--noconfirm",
                          "linux-headers", "nvidia-dkms", "nvidia-utils", "lib32-nvidia-utils",
                          "nvidia-settings", "vulkan-icd-loader", "lib32-vulkan-icd-loader",
                          "egl-wayland", "opencl-nvidia", "lib32-opencl-nvidia",
                          "libvdpau-va-gl", "libvdpau", "linux-firmware-nvidia"]));

                    commands.push(terminal::TerminalCommand::new("sudo",
                        &["sh", "-c", "sed -i 's/\\(GRUB_CMDLINE_LINUX_DEFAULT=[\"'\\''']\\)/\\1nvidia-drm.modeset=1 /' /etc/default/grub"]));
                    commands.push(terminal::TerminalCommand::new("sudo",
                        &["grub-mkconfig", "-o", "/boot/grub/grub.cfg"]));
                    commands.push(terminal::TerminalCommand::new("sudo",
                        &["sh", "-c", "sed -i 's/^MODULES=()/MODULES=(nvidia nvidia_modeset nvidia_uvm nvidia_drm)/' /etc/mkinitcpio.conf"]));
                    commands.push(terminal::TerminalCommand::new("sudo",
                        &["systemctl", "enable", "nvidia-suspend.service", "nvidia-hibernate.service", "nvidia-resume.service"]));
                    commands.push(terminal::TerminalCommand::new("sudo",
                        &["mkinitcpio", "-P"]));
                }

                if selected_ids.contains(&"nvidia_open".to_string()) {
                    commands.push(terminal::TerminalCommand::new("sudo",
                        &["pacman", "-S", "--needed", "--noconfirm",
                          "linux-headers", "nvidia-open-dkms", "nvidia-utils", "lib32-nvidia-utils",
                          "nvidia-settings", "vulkan-icd-loader", "lib32-vulkan-icd-loader",
                          "egl-wayland", "opencl-nvidia", "lib32-opencl-nvidia",
                          "libvdpau-va-gl", "libvdpau", "linux-firmware-nvidia"]));

                    commands.push(terminal::TerminalCommand::new("sudo",
                        &["sh", "-c", "sed -i 's/\\(GRUB_CMDLINE_LINUX_DEFAULT=[\"'\\''']\\)/\\1nvidia-drm.modeset=1 /' /etc/default/grub"]));
                    commands.push(terminal::TerminalCommand::new("sudo",
                        &["grub-mkconfig", "-o", "/boot/grub/grub.cfg"]));
                    commands.push(terminal::TerminalCommand::new("sudo",
                        &["sh", "-c", "sed -i 's/^MODULES=()/MODULES=(nvidia nvidia_modeset nvidia_uvm nvidia_drm)/' /etc/mkinitcpio.conf"]));
                    commands.push(terminal::TerminalCommand::new("sudo",
                        &["systemctl", "enable", "nvidia-suspend.service", "nvidia-hibernate.service", "nvidia-resume.service"]));
                    commands.push(terminal::TerminalCommand::new("sudo",
                        &["mkinitcpio", "-P"]));
                }

                if !commands.is_empty() {
                    terminal::run_terminal_commands(
                        &button_for_dialog,
                        &terminal_for_dialog,
                        &title_for_dialog,
                        commands,
                        "GPU Driver Installation",
                    );
                }
            },
        );
    }
}

