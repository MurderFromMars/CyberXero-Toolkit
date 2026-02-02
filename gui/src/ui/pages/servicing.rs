//! Servicing and system tweaks page button handlers.
//!
//! Handles:
//! - Clear Pacman cache
//! - Unlock Pacman database
//! - Plasma X11 session installation
//! - Pacman local database fix
//! - WayDroid guide
//! - Fix GPGME database
//! - Fix Arch keyring
//! - Update mirrorlist
//! - Parallel downloads adjustment

use crate::core;
use crate::ui::dialogs::selection::{
    show_selection_dialog, SelectionDialogConfig, SelectionOption, SelectionType,
};
use crate::ui::dialogs::terminal;
use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::extract_widget;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder};
use log::info;

/// Set up all button handlers for the servicing/system tweaks page
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    setup_clr_pacman(page_builder, window);
    setup_unlock_pacman(page_builder, window);
    setup_plasma_x11(page_builder, window);
    setup_pacman_db_fix(page_builder, window);
    setup_waydroid_guide(page_builder);
    setup_fix_gpgme(page_builder, window);
    setup_fix_arch_keyring(page_builder, window);
    setup_update_mirrorlist(page_builder, window);
    setup_parallel_downloads(page_builder, window);
    setup_cachyos_repos(page_builder, window);
    setup_chaotic_aur(page_builder, window);
}

fn setup_clr_pacman(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_clr_pacman = extract_widget::<gtk4::Button>(page_builder, "btn_clr_pacman");
    let window = window.clone();
    btn_clr_pacman.connect_clicked(move |_| {
        info!("Servicing: Clear Pacman Cache button clicked");
        // Use terminal dialog for interactive pacman cache clearing
        terminal::show_terminal_dialog(
            window.upcast_ref(),
            "Clear Pacman Cache",
            "pkexec",
            &["pacman", "-Scc"],
        );
    });
}

fn setup_unlock_pacman(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_unlock_pacman = extract_widget::<gtk4::Button>(page_builder, "btn_unlock_pacman");
    let window = window.clone();
    btn_unlock_pacman.connect_clicked(move |_| {
        info!("Servicing: Unlock Pacman DB button clicked");
        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .privileged()
                    .program("rm")
                    .args(&["-f", "/var/lib/pacman/db.lck"])
                    .description("Removing Pacman lock file...")
                    .build(),
            )
            .build();
        task_runner::run(window.upcast_ref(), commands, "Unlock Pacman Database");
    });
}

fn setup_plasma_x11(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_plasma_x11 = extract_widget::<gtk4::Button>(page_builder, "btn_plasma_x11");
    let window = window.clone();
    btn_plasma_x11.connect_clicked(move |_| {
        info!("Servicing: Plasma X11 Session button clicked");
        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .aur()
                    .args(&["-S", "--noconfirm", "kwin-x11", "plasma-x11-session"])
                    .description("Installing KDE Plasma X11 session components...")
                    .build(),
            )
            .build();
        task_runner::run(window.upcast_ref(), commands, "Install KDE X11 Session");
    });
}

fn setup_pacman_db_fix(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_pacman_db_fix = extract_widget::<gtk4::Button>(page_builder, "btn_pacman_db_fix");
    let window = window.clone();
    btn_pacman_db_fix.connect_clicked(move |_| {
        info!("Servicing: Pacman DB Fix button clicked");
        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .privileged()
                    .program("sh")
                    .args(&[
                        "-c",
                        "find /var/lib/pacman/local/ -type f -name 'desc' -exec sed -i '/^%INSTALLED_DB%$/,+2d' {} \\;",
                    ])
                    .description("Fixing Pacman local database...")
                    .build(),
            )
            .build();
        task_runner::run(window.upcast_ref(), commands, "Pacman DB Fix");
    });
}

fn setup_waydroid_guide(page_builder: &Builder) {
    let btn_waydroid_guide = extract_widget::<gtk4::Button>(page_builder, "btn_waydroid_guide");
    btn_waydroid_guide.connect_clicked(move |_| {
        info!("Servicing: WayDroid Guide button clicked - opening guide");
        let _ = std::process::Command::new("xdg-open")
            .arg("https://xerolinux.xyz/posts/waydroid-guide/")
            .spawn();
    });
}

fn setup_fix_gpgme(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_fix_gpgme = extract_widget::<gtk4::Button>(page_builder, "btn_fix_gpgme");
    let window = window.clone();
    btn_fix_gpgme.connect_clicked(move |_| {
        info!("Servicing: Fix GPGME Database button clicked");
        // Use terminal dialog for interactive GPGME fix
        terminal::show_terminal_dialog(
            window.upcast_ref(),
            "Fix GPGME Database",
            "pkexec",
            &["sh", "-c", "rm -rf /var/lib/pacman/sync && pacman -Syy"],
        );
    });
}

fn setup_fix_arch_keyring(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_fix_arch_keyring = extract_widget::<gtk4::Button>(page_builder, "btn_fix_arch_keyring");
    let window = window.clone();
    btn_fix_arch_keyring.connect_clicked(move |_| {
        info!("Servicing: Fix Arch Keyring button clicked");
        let commands = CommandSequence::new()
            .then(Command::builder()
                .privileged()
                .program("rm")
                .args(&["-rf", "/etc/pacman.d/gnupg"])
                .description("Removing existing GnuPG keyring...")
                .build())
            .then(Command::builder()
                .privileged()
                .program("pacman-key")
                .args(&["--init"])
                .description("Initializing new keyring...")
                .build())
            .then(Command::builder()
                .privileged()
                .program("pacman-key")
                .args(&["--populate"])
                .description("Populating keyring...")
                .build())
            .then(Command::builder()
                .privileged()
                .program("sh")
                .args(&["-c", "echo 'keyserver hkp://keyserver.ubuntu.com:80' >> /etc/pacman.d/gnupg/gpg.conf"])
                .description("Setting keyserver...")
                .build())
            .then(Command::builder()
                .privileged()
                .program("pacman")
                .args(&["-Syy", "--noconfirm", "archlinux-keyring"])
                .description("Reinstalling Arch Linux keyring...")
                .build())
            .build();
        task_runner::run(window.upcast_ref(), commands, "Fix GnuPG Keyring");
    });
}

fn setup_update_mirrorlist(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_update_mirrorlist =
        extract_widget::<gtk4::Button>(page_builder, "btn_update_mirrorlist");
    let window = window.clone();
    btn_update_mirrorlist.connect_clicked(move |_| {
        info!("Servicing: Update Mirrorlist button clicked");
        let window_ref = window.upcast_ref();

                let rate_mirrors_installed = core::is_package_installed("rate-mirrors");
                let config = SelectionDialogConfig::new(
                    "Update Mirrorlist",
                    "Select which mirrorlists to update. rate-mirrors will be installed if needed.",
                )
                .selection_type(SelectionType::Single)
                .selection_required(false)
                .add_option(SelectionOption::new(
                    "chaotic",
                    "Chaotic-AUR Mirrorlist",
                    "Also update Chaotic-AUR mirrorlist (optional)",
                    false,
                ))
                .confirm_label("Update");

                let window_for_closure = window.clone();
                show_selection_dialog(window_ref, config, move |selected_ids| {
                    let mut commands = CommandSequence::new();

                    if !rate_mirrors_installed {
                        commands = commands.then(Command::builder()
                             .aur()
                             .args(&["-S", "--needed", "--noconfirm", "rate-mirrors"])
                             .description("Installing rate-mirrors utility...")
                             .build());
                    }

                    commands = commands.then(Command::builder()
                        .privileged()
                        .program("sh")
                        .args(&["-c", "rate-mirrors --allow-root --protocol https arch | tee /etc/pacman.d/mirrorlist"])
                        .description("Updating Arch mirrorlist...")
                        .build());

                    if selected_ids.iter().any(|s| s == "chaotic") {
                        commands = commands.then(Command::builder()
                            .privileged()
                            .program("sh")
                            .args(&["-c", "rate-mirrors --allow-root --protocol https chaotic-aur | tee /etc/pacman.d/chaotic-mirrorlist"])
                            .description("Updating Chaotic-AUR mirrorlist...")
                            .build());
                    }

                    if !commands.is_empty() {
                        task_runner::run(window_for_closure.upcast_ref(), commands.build(), "Update System Mirrorlist");
                    }
                });
    });
}

fn setup_parallel_downloads(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_parallel_downloads =
        extract_widget::<gtk4::Button>(page_builder, "btn_parallel_downloads");
    let window = window.clone();
    btn_parallel_downloads.connect_clicked(move |_| {
        info!("Servicing: Change Parallel Downloads button clicked");
        // Use terminal dialog for interactive pmpd tool
        terminal::show_terminal_dialog(
            window.upcast_ref(),
            "Change Parallel Downloads",
            "pkexec",
            &["pmpd"],
        );
    });
}

fn setup_cachyos_repos(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_cachyos_repos = extract_widget::<gtk4::Button>(page_builder, "btn_cachyos_repos");
    let window = window.clone();
    btn_cachyos_repos.connect_clicked(move |_| {
        info!("Servicing: Install CachyOS Repos button clicked");
        
        // CachyOS provides an official install script
        // https://wiki.cachyos.org/configuration/general_system_tweaks/
        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .normal()
                    .program("sh")
                    .args(&[
                        "-c",
                        "curl -fsSL https://mirror.cachyos.org/cachyos-repo.tar.xz -o /tmp/cachyos-repo.tar.xz && cd /tmp && tar xvf cachyos-repo.tar.xz",
                    ])
                    .description("Downloading CachyOS repository files...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("sh")
                    .args(&[
                        "-c",
                        "cd /tmp/cachyos-repo && yes | ./cachyos-repo.sh",
                    ])
                    .description("Running CachyOS repository installer...")
                    .build(),
            )
            .then(
                Command::builder()
                    .normal()
                    .program("rm")
                    .args(&["-rf", "/tmp/cachyos-repo", "/tmp/cachyos-repo.tar.xz"])
                    .description("Cleaning up temporary files...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("pacman")
                    .args(&["-Syy"])
                    .description("Refreshing package databases...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "Install CachyOS Repositories");
    });
}

fn setup_chaotic_aur(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_chaotic_aur = extract_widget::<gtk4::Button>(page_builder, "btn_chaotic_aur");
    let window = window.clone();
    btn_chaotic_aur.connect_clicked(move |_| {
        info!("Servicing: Install Chaotic-AUR button clicked");
        
        // Chaotic-AUR official install method
        // https://aur.chaotic.cx/
        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .privileged()
                    .program("pacman-key")
                    .args(&["--recv-key", "3056513887B78AEB", "--keyserver", "keyserver.ubuntu.com"])
                    .description("Receiving Chaotic-AUR signing key...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("pacman-key")
                    .args(&["--lsign-key", "3056513887B78AEB"])
                    .description("Locally signing Chaotic-AUR key...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("pacman")
                    .args(&[
                        "-U",
                        "--noconfirm",
                        "https://cdn-mirror.chaotic.cx/chaotic-aur/chaotic-keyring.pkg.tar.zst",
                    ])
                    .description("Installing Chaotic-AUR keyring...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("pacman")
                    .args(&[
                        "-U",
                        "--noconfirm",
                        "https://cdn-mirror.chaotic.cx/chaotic-aur/chaotic-mirrorlist.pkg.tar.zst",
                    ])
                    .description("Installing Chaotic-AUR mirrorlist...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("sh")
                    .args(&[
                        "-c",
                        "grep -q '\\[chaotic-aur\\]' /etc/pacman.conf || echo -e '\\n[chaotic-aur]\\nInclude = /etc/pacman.d/chaotic-mirrorlist' >> /etc/pacman.conf",
                    ])
                    .description("Adding Chaotic-AUR to pacman.conf...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("pacman")
                    .args(&["-Syy"])
                    .description("Refreshing package databases...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "Install Chaotic-AUR Repository");
    });
}
fn setup_xero_repo(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_xero_repo = extract_widget::<gtk4::Button>(page_builder, "btn_xero_repo");
    let window = window.clone();
    btn_xero_repo.connect_clicked(move |_| {
        info!("Servicing: Add Xero Linux Repository button clicked");
        
        // Xero Linux repository setup
        // https://repos.xerolinux.xyz/
        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .privileged()
                    .program("sh")
                    .args(&[
                        "-c",
                        "grep -q '\\[xerolinux\\]' /etc/pacman.conf || echo -e '\\n[xerolinux]\\nSigLevel = Optional TrustAll\\nServer = https://repos.xerolinux.xyz/$repo/$arch' >> /etc/pacman.conf",
                    ])
                    .description("Adding Xero Linux repository to pacman.conf...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("pacman")
                    .args(&["-Syy"])
                    .description("Refreshing package databases...")
                    .build(),
            )
            .build();

        task_runner::run(window.upcast_ref(), commands, "Add Xero Linux Repository");
    });
}
