//! Customization page button handlers.
//!
//! Handles:
//! - CyberXero Theme installation
//! - PS4 Theme installation
//! - ZSH All-in-One setup
//! - Save Desktop tool
//! - GRUB theme installation
//! - Plymouth Manager
//! - Update Layan Theme
//! - Decky Loader management (install/update/uninstall/wipe)
//! - Config/Rice reset

use crate::ui::dialogs::terminal;
use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::extract_widget;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button};
use log::info;

/// Set up all button handlers for the customization page.
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    setup_cyberxero_theme(page_builder, window);
    setup_ps4_theme(page_builder, window);
    setup_zsh_aio(page_builder, window);
    setup_save_desktop(page_builder, window);
    setup_grub_theme(page_builder, window);
    setup_plymouth_manager(page_builder, window);
    setup_layan_patch(page_builder, window);
    setup_decky_loader(page_builder, window);
    setup_config_reset(page_builder, window);
}

fn setup_cyberxero_theme(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_cyberxero_theme");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("CyberXero Theme button clicked");

        let window_clone = window.clone();
        crate::ui::dialogs::warning::show_warning_confirmation(
            window.upcast_ref(),
            "Apply CyberXero Theme",
            "This will install the <span foreground=\"cyan\" weight=\"bold\">CyberXero Dynamic Tiling Theme</span>.\n\n\
             • Existing Plasma configs will be <span foreground=\"cyan\" weight=\"bold\">backed up</span> automatically\n\
             • KWin effects will be compiled from source\n\
             • Plasmashell will be <span foreground=\"red\" weight=\"bold\">restarted</span> during installation\n\n\
             This process may take several minutes.",
            move || {
                terminal::show_terminal_dialog(
                    window_clone.upcast_ref(),
                    "CyberXero Theme Installation",
                    "/usr/local/bin/cyberxero-theme",
                    &[],
                );
            },
        );
    });
}

fn setup_ps4_theme(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_ps4_theme");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("PS4 Theme button clicked");

        let window_clone = window.clone();
        crate::ui::dialogs::warning::show_warning_confirmation(
            window.upcast_ref(),
            "Apply PS4 Theme",
            "This will install the <span foreground=\"cyan\" weight=\"bold\">PlayStation 4 Plasma Theme</span>.\n\n\
             • Existing Plasma configs will be <span foreground=\"cyan\" weight=\"bold\">backed up</span> automatically\n\
             • KWin effects will be compiled from source\n\
             • Plasmashell will be <span foreground=\"red\" weight=\"bold\">restarted</span> during installation\n\n\
             This process may take several minutes.",
            move || {
                terminal::show_terminal_dialog(
                    window_clone.upcast_ref(),
                    "PS4 Theme Installation",
                    "/usr/local/bin/PS4-theme",
                    &[],
                );
            },
        );
    });
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
                .normal()
                .program("sh")
                .args(&[
                    "-c",
                    "curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh | sh -s -- --unattended",
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
                .normal()
                .program("sh")
                .args(&[
                    "-c",
                    &format!(
                        "sed -i 's|Command=/bin/bash|Command=/bin/zsh|g' {}/.local/share/konsole/XeroLinux.profile 2>/dev/null || true",
                        home
                    ),
                ])
                .description("Updating Konsole profile to use ZSH...")
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

fn setup_decky_loader(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_decky_loader");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Decky Loader button clicked");

        let window_clone = window.clone();
        let env = crate::config::env::get();
        let home = env.home.clone();

        // Check if Decky is currently installed
        let plugin_loader_path = format!("{}/homebrew/services/PluginLoader", home);
        let is_installed = std::path::Path::new(&plugin_loader_path).exists();

        let (title, description) = if is_installed {
            ("Decky Loader", "Decky Loader is currently installed, select an action")
        } else {
            ("Decky Loader", "Setup Decky loader, select a branch to install")
        };

        let mut config = crate::ui::dialogs::selection::SelectionDialogConfig::new(title, description)
            .selection_type(crate::ui::dialogs::selection::SelectionType::Single)
            .confirm_label("Continue");

        if is_installed {
            config = config
                .add_option(crate::ui::dialogs::selection::SelectionOption::new(
                    "update_release",
                    "Update to Latest Release",
                    "Recommended for stable Steam client",
                    false,
                ))
                .add_option(crate::ui::dialogs::selection::SelectionOption::new(
                    "update_prerelease",
                    "Update to Latest Pre-Release",
                    "Recommended for beta Steam client",
                    false,
                ))
                .add_option(crate::ui::dialogs::selection::SelectionOption::new(
                    "uninstall",
                    "Uninstall Decky Loader",
                    "Remove Decky Loader but keep plugins and config intact",
                    false,
                ))
                .add_option(crate::ui::dialogs::selection::SelectionOption::new(
                    "wipe",
                    "Wipe Decky Loader",
                    "Completely remove Decky Loader including all plugins and config",
                    false,
                ));
        } else {
            config = config
                .add_option(crate::ui::dialogs::selection::SelectionOption::new(
                    "install_release",
                    "Install Latest Release",
                    "Recommended for stable SteamOS",
                    false,
                ))
                .add_option(crate::ui::dialogs::selection::SelectionOption::new(
                    "install_prerelease",
                    "Install Latest Pre-Release",
                    "Recommended for beta/preview SteamOS",
                    false,
                ));
        }

        crate::ui::dialogs::selection::show_selection_dialog(
            window.upcast_ref(),
            config,
            move |selected| {
                if let Some(action) = selected.first() {
                    let home = crate::config::env::get().home.clone();

                    match action.as_str() {
                        "install_release" | "update_release" => {
                            terminal::show_terminal_dialog(
                                window_clone.upcast_ref(),
                                "Decky Loader — Install Release",
                                "sh",
                                &[
                                    "-c",
                                    concat!(
                                        "curl -L https://github.com/SteamDeckHomebrew/decky-installer/releases/latest/download/install_release.sh ",
                                        "--connect-timeout 60 -o /tmp/decky_install_release.sh && ",
                                        "chmod +x /tmp/decky_install_release.sh && ",
                                        "sh /tmp/decky_install_release.sh; ",
                                        "rm -f /tmp/decky_install_release.sh; ",
                                        "echo ''; echo 'Done! Return to Gaming Mode to use Decky Loader.'; ",
                                        "echo 'Press Enter to close...'; read"
                                    ),
                                ],
                            );
                        }
                        "install_prerelease" | "update_prerelease" => {
                            terminal::show_terminal_dialog(
                                window_clone.upcast_ref(),
                                "Decky Loader — Install Pre-Release",
                                "sh",
                                &[
                                    "-c",
                                    concat!(
                                        "curl -L https://github.com/SteamDeckHomebrew/decky-installer/releases/latest/download/install_prerelease.sh ",
                                        "--connect-timeout 60 -o /tmp/decky_install_prerelease.sh && ",
                                        "chmod +x /tmp/decky_install_prerelease.sh && ",
                                        "sh /tmp/decky_install_prerelease.sh; ",
                                        "rm -f /tmp/decky_install_prerelease.sh; ",
                                        "echo ''; echo 'Done! Return to Gaming Mode to use Decky Loader.'; ",
                                        "echo 'Press Enter to close...'; read"
                                    ),
                                ],
                            );
                        }
                        "uninstall" => {
                            let window_inner = window_clone.clone();
                            crate::ui::dialogs::warning::show_warning_confirmation(
                                window_clone.upcast_ref(),
                                "Uninstall Decky Loader",
                                "This will <span foreground=\"red\" weight=\"bold\">remove</span> Decky Loader services.\n\n\
                                 Your plugins and configuration in <span foreground=\"cyan\" weight=\"bold\">~/homebrew</span> will be <span foreground=\"cyan\" weight=\"bold\">kept intact</span>.\n\n\
                                 CEF remote debugging will be disabled.",
                                move || {
                                    let homebrew = format!("{}/homebrew", home);
                                    let cef_path = format!("{}/.steam/steam/.cef-enable-remote-debugging", home);
                                    let cef_flatpak = format!("{}/.var/app/com.valvesoftware.Steam/data/Steam/.cef-enable-remote-debugging", home);

                                    let commands = CommandSequence::new()
                                        .then(Command::builder()
                                            .privileged()
                                            .program("systemctl")
                                            .args(&["disable", "--now", "plugin_loader.service"])
                                            .description("Disabling and stopping Decky Loader service...")
                                            .build())
                                        .then(Command::builder()
                                            .privileged()
                                            .program("bash")
                                            .args(&["-c", &format!(
                                                "rm -f /etc/systemd/system/plugin_loader.service; \
                                                 rm -f {}/.config/systemd/user/plugin_loader.service",
                                                home
                                            )])
                                            .description("Removing service files...")
                                            .build())
                                        .then(Command::builder()
                                            .normal()
                                            .program("bash")
                                            .args(&["-c", "rm -rf /tmp/plugin_loader /tmp/user_install_script.sh"])
                                            .description("Cleaning up temporary files...")
                                            .build())
                                        .then(Command::builder()
                                            .privileged()
                                            .program("rm")
                                            .args(&["-f", &format!("{}/services/PluginLoader", homebrew)])
                                            .description("Removing Decky Loader binary...")
                                            .build())
                                        .then(Command::builder()
                                            .privileged()
                                            .program("bash")
                                            .args(&["-c", &format!(
                                                "rm -f '{}' '{}' 2>/dev/null; true",
                                                cef_path, cef_flatpak
                                            )])
                                            .description("Disabling CEF remote debugging...")
                                            .build())
                                        .build();

                                    task_runner::run(
                                        window_inner.upcast_ref(),
                                        commands,
                                        "Uninstall Decky Loader",
                                    );
                                },
                            );
                        }
                        "wipe" => {
                            let window_inner = window_clone.clone();
                            crate::ui::dialogs::warning::show_warning_confirmation(
                                window_clone.upcast_ref(),
                                "Wipe Decky Loader",
                                "<span foreground=\"red\" weight=\"bold\">WARNING: This is a destructive action!</span>\n\n\
                                 This will <span foreground=\"red\" weight=\"bold\">completely remove</span> Decky Loader <span foreground=\"red\" weight=\"bold\">including all plugins and configuration</span>.\n\n\
                                 The entire <span foreground=\"cyan\" weight=\"bold\">~/homebrew</span> folder will be deleted.\n\
                                 CEF remote debugging will be disabled.\n\n\
                                 This action <span foreground=\"red\" weight=\"bold\">cannot be undone</span>.",
                                move || {
                                    let homebrew = format!("{}/homebrew", home);
                                    let cef_path = format!("{}/.steam/steam/.cef-enable-remote-debugging", home);
                                    let cef_flatpak = format!("{}/.var/app/com.valvesoftware.Steam/data/Steam/.cef-enable-remote-debugging", home);

                                    let commands = CommandSequence::new()
                                        .then(Command::builder()
                                            .privileged()
                                            .program("systemctl")
                                            .args(&["disable", "--now", "plugin_loader.service"])
                                            .description("Disabling and stopping Decky Loader service...")
                                            .build())
                                        .then(Command::builder()
                                            .privileged()
                                            .program("bash")
                                            .args(&["-c", &format!(
                                                "rm -f /etc/systemd/system/plugin_loader.service; \
                                                 rm -f {}/.config/systemd/user/plugin_loader.service",
                                                home
                                            )])
                                            .description("Removing service files...")
                                            .build())
                                        .then(Command::builder()
                                            .normal()
                                            .program("bash")
                                            .args(&["-c", "rm -rf /tmp/plugin_loader /tmp/user_install_script.sh"])
                                            .description("Cleaning up temporary files...")
                                            .build())
                                        .then(Command::builder()
                                            .privileged()
                                            .program("rm")
                                            .args(&["-rf", &homebrew])
                                            .description("Deleting entire homebrew folder...")
                                            .build())
                                        .then(Command::builder()
                                            .privileged()
                                            .program("bash")
                                            .args(&["-c", &format!(
                                                "rm -f '{}' '{}' 2>/dev/null; true",
                                                cef_path, cef_flatpak
                                            )])
                                            .description("Disabling CEF remote debugging...")
                                            .build())
                                        .build();

                                    task_runner::run(
                                        window_inner.upcast_ref(),
                                        commands,
                                        "Wipe Decky Loader",
                                    );
                                },
                            );
                        }
                        _ => {}
                    }
                }
            },
        );
    });
}

fn setup_config_reset(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_config_reset");
    let window = window.clone();

    button.connect_clicked(move |_| {
        info!("Config/Rice Reset button clicked");

        let window_clone = window.clone();
        crate::ui::dialogs::warning::show_warning_confirmation(
            window.upcast_ref(),
            "Config/Rice Reset",
            "A backup of <span foreground=\"cyan\" weight=\"bold\">~/.config</span> will be created.\n\
             Once reset, the system will <span foreground=\"red\" weight=\"bold\">reboot</span>.\n\n\
             You will be getting updated config as of reset time.",
            move || {
                let commands = CommandSequence::new()
                    .then(
                        Command::builder()
                            .normal()
                            .program("bash")
                            .args(&[
                                "-c",
                                "cp -Rf ~/.config ~/.config-backup-$(date +%Y.%m.%d-%H.%M.%S)",
                            ])
                            .description("Backing up configuration...")
                            .build(),
                    )
                    .then(
                        Command::builder()
                            .normal()
                            .program("bash")
                            .args(&["-c", "cp -Rf /etc/skel/. ~"])
                            .description("Restoring default configuration...")
                            .build(),
                    )
                    .then(
                        Command::builder()
                            .normal()
                            .program("reboot")
                            .description("Rebooting system...")
                            .build(),
                    )
                    .build();

                task_runner::run(
                    window_clone.upcast_ref(),
                    commands,
                    "Config/Rice Reset",
                );
            },
        );
    });
}
