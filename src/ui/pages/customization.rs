//! Customization page button handlers.
//!
//! Handles:
//! - ZSH All-in-One setup
//! - Save Desktop tool
//! - GRUB theme installation
//! - Plasma wallpapers
//! - Layan GTK4 patch

use crate::ui::terminal;
use crate::utils;
use gtk4::prelude::*;
use gtk4::{Builder, Label};
use log::{info, warn};
use vte4::prelude::*;
use vte4::Terminal;

/// Set up all button handlers for the customization page
pub fn setup_handlers(page_builder: &Builder, main_builder: &Builder) {
    let terminal_output: Terminal = main_builder
        .object("global_terminal_output_view")
        .expect("Failed to get terminal output view");
    let terminal_title: Label = main_builder
        .object("global_terminal_title")
        .expect("Failed to get terminal title label");

    setup_zsh_aio(&page_builder, &terminal_output, &terminal_title);
    setup_save_desktop(&page_builder, &terminal_output, &terminal_title);
    setup_grub_theme(&page_builder, &terminal_output, &terminal_title);
    setup_wallpapers(&page_builder, &terminal_output, &terminal_title);
    setup_layan_patch(&page_builder, &terminal_output, &terminal_title);
}

fn setup_zsh_aio(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_zsh_aio) = page_builder.object::<gtk4::Button>("btn_zsh_aio") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_zsh_aio.connect_clicked(move |button| {
            info!("Customization: ZSH AiO button clicked");

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

            let mut commands = vec![];
            commands.push(terminal::TerminalCommand::new(helper,
                &["-S", "--needed", "--noconfirm", "zsh", "grml-zsh-config", "fastfetch"]));
            commands.push(terminal::TerminalCommand::new("sh",
                &["-c", "sh -c \"$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)\" \"\" --unattended"]));
            commands.push(terminal::TerminalCommand::new(helper,
                &["-S", "--noconfirm", "--needed", "pacseek", "ttf-meslo-nerd", "siji-git",
                  "otf-unifont", "bdf-unifont", "noto-color-emoji-fontconfig", "xorg-fonts-misc",
                  "ttf-dejavu", "ttf-meslo-nerd-font-powerlevel10k", "noto-fonts-emoji",
                  "powerline-fonts", "oh-my-posh-bin"]));

            let home = std::env::var("HOME").unwrap_or_default();
            commands.push(terminal::TerminalCommand::new("git",
                &["clone", "https://github.com/zsh-users/zsh-completions",
                  &format!("{}/.oh-my-zsh/custom/plugins/zsh-completions", home)]));
            commands.push(terminal::TerminalCommand::new("git",
                &["clone", "https://github.com/zsh-users/zsh-autosuggestions",
                  &format!("{}/.oh-my-zsh/custom/plugins/zsh-autosuggestions", home)]));
            commands.push(terminal::TerminalCommand::new("git",
                &["clone", "https://github.com/zsh-users/zsh-syntax-highlighting.git",
                  &format!("{}/.oh-my-zsh/custom/plugins/zsh-syntax-highlighting", home)]));
            commands.push(terminal::TerminalCommand::new("sh",
                &["-c", &format!("mv -f {}/.zshrc {}/.zshrc.user 2>/dev/null || true", home, home)]));
            commands.push(terminal::TerminalCommand::new("wget",
                &["-q", "-P", &home, "https://raw.githubusercontent.com/xerolinux/xero-fixes/main/conf/.zshrc"]));
            commands.push(terminal::TerminalCommand::new("sudo",
                &["chsh", &std::env::var("USER").unwrap_or_default(), "-s", "/bin/zsh"]));

            terminal::run_terminal_commands(
                button,
                &terminal_clone,
                &title_clone,
                commands,
                "ZSH All-in-One Setup",
            );
        });
    }
}

fn setup_save_desktop(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_save_desktop) = page_builder.object::<gtk4::Button>("btn_save_desktop") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_save_desktop.connect_clicked(move |button| {
            info!("Customization: Save Desktop Tool button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(b"\r\nAnother action is already running. Please wait for it to complete.\r\n");
                return;
            }

            let commands = vec![
                terminal::TerminalCommand::new("flatpak",
                    &["install", "-y", "io.github.vikdevelop.SaveDesktop"]),
            ];

            terminal::run_terminal_commands(
                button,
                &terminal_clone,
                &title_clone,
                commands,
                "Save Desktop Tool Installation",
            );
        });
    }
}

fn setup_grub_theme(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_grub_theme) = page_builder.object::<gtk4::Button>("btn_grub_theme") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_grub_theme.connect_clicked(move |button| {
            info!("Customization: GRUB Theme button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(b"\r\nAnother action is already running. Please wait for it to complete.\r\n");
                return;
            }

            let home = std::env::var("HOME").unwrap_or_default();
            let commands = vec![
                terminal::TerminalCommand::new("git",
                    &["clone", "--depth", "1", "https://github.com/xerolinux/xero-grubs",
                      &format!("{}/xero-grubs", home)]),
                terminal::TerminalCommand::new("sh",
                    &["-c", &format!("cd {}/xero-grubs && sh install.sh", home)]),
                terminal::TerminalCommand::new("rm",
                    &["-rf", &format!("{}/xero-grubs", home)]),
            ];

            terminal::run_terminal_commands(
                button,
                &terminal_clone,
                &title_clone,
                commands,
                "XeroLinux GRUB Theme Installation",
            );
        });
    }
}

fn setup_wallpapers(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_wallpapers) = page_builder.object::<gtk4::Button>("btn_wallpapers") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_wallpapers.connect_clicked(move |button| {
            info!("Customization: Plasma Wallpapers button clicked");

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
                terminal::TerminalCommand::new(helper,
                    &["-S", "--noconfirm", "--needed", "kde-wallpapers-extra"]),
            ];

            terminal::run_terminal_commands(
                button,
                &terminal_clone,
                &title_clone,
                commands,
                "Plasma Wallpapers Installation (~1.2GB)",
            );
        });
    }
}

fn setup_layan_patch(page_builder: &Builder, terminal: &Terminal, terminal_title: &Label) {
    if let Some(btn_layan_patch) = page_builder.object::<gtk4::Button>("btn_layan_patch") {
        let terminal_clone = terminal.clone();
        let title_clone = terminal_title.clone();

        btn_layan_patch.connect_clicked(move |button| {
            info!("Customization: Layan GTK4 Patch button clicked");

            if terminal::is_action_running() {
                warn!("Action already running");
                terminal_clone.feed(b"\r\nAnother action is already running. Please wait for it to complete.\r\n");
                return;
            }

            let home = std::env::var("HOME").unwrap_or_default();
            let commands = vec![
                terminal::TerminalCommand::new("git",
                    &["clone", "--depth", "1", "https://github.com/vinceliuice/Layan-gtk-theme.git",
                      &format!("{}/Layan-gtk-theme", home)]),
                terminal::TerminalCommand::new("sh",
                    &["-c", &format!("cd {}/Layan-gtk-theme && sh install.sh -l -c dark -d {}/.themes", home, home)]),
                terminal::TerminalCommand::new("rm",
                    &["-rf", &format!("{}/Layan-gtk-theme", home)]),
                terminal::TerminalCommand::new("git",
                    &["clone", "--depth", "1", "https://github.com/vinceliuice/Layan-kde.git",
                      &format!("{}/Layan-kde", home)]),
                terminal::TerminalCommand::new("sh",
                    &["-c", &format!("cd {}/Layan-kde && sh install.sh", home)]),
                terminal::TerminalCommand::new("rm",
                    &["-rf", &format!("{}/Layan-kde", home)]),
            ];

            terminal::run_terminal_commands(
                button,
                &terminal_clone,
                &title_clone,
                commands,
                "Layan GTK4 Patch & Update",
            );
        });
    }
}
