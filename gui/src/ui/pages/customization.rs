//! Customization page button handlers.
//!
//! Handles:
//! - ZSH All-in-One setup
//! - Save Desktop tool
//! - GRUB theme installation
//! - Plymouth Manager
//! - Update Layan Theme

use crate::ui::dialogs::terminal;
use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::extract_widget;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button};
use log::info;

/// Set up all button handlers for the customization page.
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    setup_zsh_aio(page_builder, window);
    setup_save_desktop(page_builder, window);
    setup_grub_theme(page_builder, window);
    setup_plymouth_manager(page_builder, window);
    setup_layan_patch(page_builder, window);
}

fn setup_zsh_aio(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_zsh_aio");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("ZSH AiO button clicked");

        let env = crate::config::env::get();
        let home = env.home.clone();
        let user = env.user.clone();

        let commands = CommandSequence::new()
            .then(Command::builder()
                .aur()
                .args(&[
                    "-S",
                    "--needed",
                    "--noconfirm",
                    "zsh",
                    "grml-zsh-config",
                    "fastfetch",
                ])
                .description("Installing ZSH and dependencies...")
                .build())
            .then(Command::builder()
                .privileged()
                .program("sh")
                .args(&[
                    "-c",
                    "sh -c \"$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)\" \"\" --unattended",
                ])
                .description("Installing Oh My Zsh framework...")
                .build())
            .then(Command::builder()
                .aur()
                .args(&[
                    "-S",
                    "--noconfirm",
                    "--needed",
                    "pacseek",
                    "ttf-meslo-nerd",
                    "siji-git",
                    "otf-unifont",
                    "bdf-unifont",
                    "noto-color-emoji-fontconfig",
                    "xorg-fonts-misc",
                    "ttf-dejavu",
                    "ttf-meslo-nerd-font-powerlevel10k",
                    "noto-fonts-emoji",
                    "powerline-fonts",
                    "oh-my-posh-bin",
                ])
                .description("Installing fonts and terminal enhancements...")
                .build())
            .then(Command::builder()
                .normal()
                .program("git")
                .args(&[
                    "clone",
                    "https://github.com/zsh-users/zsh-completions",
                    &format!("{}/.oh-my-zsh/custom/plugins/zsh-completions", home),
                ])
                .description("Installing ZSH completions plugin...")
                .build())
            .then(Command::builder()
                .normal()
                .program("git")
                .args(&[
                    "clone",
                    "https://github.com/zsh-users/zsh-autosuggestions",
                    &format!("{}/.oh-my-zsh/custom/plugins/zsh-autosuggestions", home),
                ])
                .description("Installing ZSH autosuggestions plugin...")
                .build())
            .then(Command::builder()
                .normal()
                .program("git")
                .args(&[
                    "clone",
                    "https://github.com/zsh-users/zsh-syntax-highlighting.git",
                    &format!("{}/.oh-my-zsh/custom/plugins/zsh-syntax-highlighting", home),
                ])
                .description("Installing ZSH syntax highlighting plugin...")
                .build())
            .then(Command::builder()
                .normal()
                .program("sh")
                .args(&[
                    "-c",
                    &format!(
                        "mv -f {}/.zshrc {}/.zshrc.user 2>/dev/null || true",
                        home, home
                    ),
                ])
                .description("Backing up existing ZSH configuration...")
                .build())
            .then(Command::builder()
                .normal()
                .program("wget")
                .args(&[
                    "-q",
                    "-P",
                    &home,
                    "https://raw.githubusercontent.com/xerolinux/xero-fixes/main/conf/.zshrc",
                ])
                .description("Downloading XeroLinux ZSH configuration...")
                .build())
            .then(Command::builder()
                .privileged()
                .program("chsh")
                .args(&[&user, "-s", "/bin/zsh"])
                .description("Setting ZSH as default shell...")
                .build())
            .build();

        task_runner::run(
            window.upcast_ref(),
            commands,
            "ZSH All-in-One Setup",
        );
    });
}

fn setup_save_desktop(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_save_desktop");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Save Desktop Tool button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .normal()
                    .program("flatpak")
                    .args(&["install", "-y", "io.github.vikdevelop.SaveDesktop"])
                    .description("Installing Save Desktop tool from Flathub...")
                    .build(),
            )
            .build();

        task_runner::run(
            window.upcast_ref(),
            commands,
            "Save Desktop Tool Installation",
        );
    });
}

fn setup_grub_theme(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_grub_theme");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("GRUB Theme button clicked");

        let home = crate::config::env::get().home.clone();
        let repo_path = format!("{}/xero-grubs", home);

        // Run everything in terminal - clone if needed, then run interactive installation script
        let install_command = format!(
            "if [ ! -d \"{}\" ]; then git clone --depth 1 https://github.com/xerolinux/xero-grubs \"{}\"; fi && pkexec sh -c 'cd \"{}\" && ./install.sh'",
            repo_path, repo_path, repo_path
        );

        terminal::show_terminal_dialog(
            window.upcast_ref(),
            "XeroLinux GRUB Theme Installation",
            "sh",
            &["-c", &install_command],
        );
    });
}

fn setup_plymouth_manager(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_plymouth_manager");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Plymouth Manager button clicked");

        terminal::show_terminal_dialog(
            window.upcast_ref(),
            "Plymouth Manager",
            "/usr/local/bin/xpm",
            &[],
        );
    });
}

fn setup_layan_patch(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_layan_patch");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Update Layan Theme button clicked");

        let home = crate::config::env::get().home.clone();

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .normal()
                    .program("git")
                    .args(&[
                        "clone",
                        "--depth",
                        "1",
                        "https://github.com/vinceliuice/Layan-kde.git",
                        &format!("{}/Layan-kde", home),
                    ])
                    .description("Downloading Layan KDE theme...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("sh")
                    .args(&["-c", &format!("cd {}/Layan-kde && sh install.sh", home)])
                    .description("Installing Layan KDE theme...")
                    .build(),
            )
            .then(
                Command::builder()
                    .normal()
                    .program("rm")
                    .args(&["-rf", &format!("{}/Layan-kde", home)])
                    .description("Cleaning up KDE theme files...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "Update Layan Theme");
    });
}
