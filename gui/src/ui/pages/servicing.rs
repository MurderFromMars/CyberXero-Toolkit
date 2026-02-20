//! Servicing and system tweaks page button handlers.

use adw::prelude::*;
use crate::config;
use crate::core;
use crate::ui::dialogs::terminal;
use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::extract_widget;
use gtk4::{
    ApplicationWindow, Box as GtkBox, Builder, CheckButton, Frame, Label, Orientation,
    ScrolledWindow, Separator,
};
use log::info;
use std::cell::RefCell;
use std::rc::Rc;

pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    setup_clr_pacman(page_builder, window);
    setup_unlock_pacman(page_builder, window);
    setup_remove_orphans(page_builder, window);
    setup_plasma_x11(page_builder, window);
    setup_pacman_db_fix(page_builder, window);
    setup_waydroid_guide(page_builder);
    setup_fix_gpgme(page_builder, window);
    setup_fix_arch_keyring(page_builder, window);
    setup_update_mirrorlist(page_builder, window);
    setup_parallel_downloads(page_builder, window);
    setup_cachyos_repos(page_builder, window);
    setup_chaotic_aur(page_builder, window);
    setup_xero_repo(page_builder, window);
    setup_xpackagemanager(page_builder, window);
    setup_update_toolkit(page_builder, window);
}

fn setup_clr_pacman(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_clr_pacman = extract_widget::<gtk4::Button>(page_builder, "btn_clr_pacman");
    let window = window.clone();
    btn_clr_pacman.connect_clicked(move |_| {
        info!("Servicing: Clear Pacman Cache button clicked");
        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .privileged()
                    .program("pacman")
                    .args(&["-Scc", "--noconfirm"])
                    .description("Clearing Pacman cache...")
                    .build(),
            )
            .build();
        task_runner::run(window.upcast_ref(), commands, "Clear Pacman Cache");
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

/// Query pacman for orphaned packages (installed as deps, no longer required).
fn get_orphan_packages() -> Vec<String> {
    std::process::Command::new("pacman")
        .args(["-Qdtq"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| l.to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn setup_remove_orphans(page_builder: &Builder, window: &ApplicationWindow) {
    let btn = extract_widget::<gtk4::Button>(page_builder, "btn_remove_orphans");
    let window = window.clone();

    btn.connect_clicked(move |_| {
        info!("Servicing: Remove Orphans button clicked");

        let orphans = get_orphan_packages();

        if orphans.is_empty() {
            // No orphans — show a simple info dialog
            let dialog = adw::Window::new();
            dialog.set_title(Some("Xero Toolkit - Remove Orphans"));
            dialog.set_default_size(400, 200);
            dialog.set_modal(true);
            dialog.set_transient_for(Some(&window));

            let toolbar = adw::ToolbarView::new();
            let header = adw::HeaderBar::new();
            toolbar.add_top_bar(&header);

            let content = GtkBox::new(Orientation::Vertical, 16);
            content.set_margin_top(24);
            content.set_margin_bottom(24);
            content.set_margin_start(24);
            content.set_margin_end(24);
            content.set_halign(gtk4::Align::Center);
            content.set_valign(gtk4::Align::Center);

            let icon = gtk4::Image::from_icon_name("emblem-ok-symbolic");
            icon.set_pixel_size(48);
            content.append(&icon);

            let label = Label::new(Some("No orphaned packages found.\nYour system is clean!"));
            label.set_halign(gtk4::Align::Center);
            label.set_justify(gtk4::Justification::Center);
            content.append(&label);

            let ok_btn = gtk4::Button::with_label("OK");
            ok_btn.add_css_class("suggested-action");
            ok_btn.add_css_class("pill");
            ok_btn.set_halign(gtk4::Align::Center);
            let dialog_clone = dialog.clone();
            ok_btn.connect_clicked(move |_| dialog_clone.close());
            content.append(&ok_btn);

            toolbar.set_content(Some(&content));
            dialog.set_content(Some(&toolbar));
            dialog.present();
            return;
        }

        // ── Build the orphan review dialog ───────────────────────────────
        let dialog = adw::Window::new();
        dialog.set_title(Some("Xero Toolkit - Remove Orphans"));
        dialog.set_default_size(550, 500);
        dialog.set_modal(true);
        dialog.set_transient_for(Some(&window));

        let toolbar = adw::ToolbarView::new();
        let header = adw::HeaderBar::new();
        toolbar.add_top_bar(&header);

        let outer = GtkBox::new(Orientation::Vertical, 12);
        outer.set_margin_top(12);
        outer.set_margin_bottom(12);
        outer.set_margin_start(12);
        outer.set_margin_end(12);

        // Title + description
        let title_box = GtkBox::new(Orientation::Vertical, 4);
        title_box.set_halign(gtk4::Align::Center);

        let title = Label::new(Some("Remove Orphaned Packages"));
        title.add_css_class("title-2");
        title_box.append(&title);

        let count_text = format!(
            "Found {} orphaned package{}. Uncheck any you want to keep.",
            orphans.len(),
            if orphans.len() == 1 { "" } else { "s" }
        );
        let subtitle = Label::new(Some(&count_text));
        subtitle.add_css_class("dim-label");
        subtitle.set_wrap(true);
        subtitle.set_halign(gtk4::Align::Center);
        title_box.append(&subtitle);

        outer.append(&title_box);

        // Select All / Deselect All row
        let toggle_row = GtkBox::new(Orientation::Horizontal, 8);
        toggle_row.set_halign(gtk4::Align::End);
        toggle_row.set_margin_end(24);

        let btn_select_all = gtk4::Button::with_label("Select All");
        btn_select_all.add_css_class("flat");
        btn_select_all.add_css_class("caption");
        toggle_row.append(&btn_select_all);

        let btn_deselect_all = gtk4::Button::with_label("Deselect All");
        btn_deselect_all.add_css_class("flat");
        btn_deselect_all.add_css_class("caption");
        toggle_row.append(&btn_deselect_all);

        outer.append(&toggle_row);

        // Scrollable package list inside a frame
        let frame = Frame::new(None);
        frame.add_css_class("view");
        frame.set_hexpand(true);
        frame.set_vexpand(true);
        frame.set_margin_start(24);
        frame.set_margin_end(24);
        frame.set_margin_top(4);
        frame.set_margin_bottom(8);

        let scroll = ScrolledWindow::new();
        scroll.set_hexpand(true);
        scroll.set_vexpand(true);
        scroll.set_min_content_height(250);

        let list_box = GtkBox::new(Orientation::Vertical, 0);
        list_box.set_margin_start(16);
        list_box.set_margin_end(16);
        list_box.set_margin_top(8);
        list_box.set_margin_bottom(8);

        let checkboxes: Rc<RefCell<Vec<(String, CheckButton)>>> =
            Rc::new(RefCell::new(Vec::new()));

        for (i, pkg) in orphans.iter().enumerate() {
            let row = GtkBox::new(Orientation::Horizontal, 12);
            row.set_margin_top(4);
            row.set_margin_bottom(4);

            let checkbox = CheckButton::new();
            checkbox.set_active(true); // pre-checked for removal
            row.append(&checkbox);

            let label = Label::new(Some(pkg));
            label.set_halign(gtk4::Align::Start);
            label.set_hexpand(true);
            label.add_css_class("monospace");
            row.append(&label);

            list_box.append(&row);
            checkboxes.borrow_mut().push((pkg.clone(), checkbox));

            if i < orphans.len() - 1 {
                let sep = Separator::new(Orientation::Horizontal);
                list_box.append(&sep);
            }
        }

        scroll.set_child(Some(&list_box));
        frame.set_child(Some(&scroll));
        outer.append(&frame);

        // Select All / Deselect All logic
        let cbs = checkboxes.clone();
        btn_select_all.connect_clicked(move |_| {
            for (_, cb) in cbs.borrow().iter() {
                cb.set_active(true);
            }
        });

        let cbs = checkboxes.clone();
        btn_deselect_all.connect_clicked(move |_| {
            for (_, cb) in cbs.borrow().iter() {
                cb.set_active(false);
            }
        });

        // Update remove button label with count
        let remove_btn = gtk4::Button::with_label(&format!("Remove {}", orphans.len()));
        remove_btn.add_css_class("destructive-action");
        remove_btn.add_css_class("pill");

        let cbs = checkboxes.clone();
        let remove_btn_clone = remove_btn.clone();
        let update_count = move || {
            let count = cbs.borrow().iter().filter(|(_, cb)| cb.is_active()).count();
            if count > 0 {
                remove_btn_clone.set_label(&format!("Remove {}", count));
                remove_btn_clone.set_sensitive(true);
            } else {
                remove_btn_clone.set_label("Remove");
                remove_btn_clone.set_sensitive(false);
            }
        };

        // Connect each checkbox toggle to update the count
        for (_, cb) in checkboxes.borrow().iter() {
            let update = update_count.clone();
            cb.connect_toggled(move |_| update());
        }

        // Button row
        let btn_row = GtkBox::new(Orientation::Horizontal, 8);
        btn_row.set_halign(gtk4::Align::Center);
        btn_row.set_margin_top(12);

        let cancel_btn = gtk4::Button::with_label("Cancel");
        cancel_btn.add_css_class("pill");
        let dialog_clone = dialog.clone();
        cancel_btn.connect_clicked(move |_| {
            info!("Orphan removal cancelled");
            dialog_clone.close();
        });

        btn_row.append(&cancel_btn);
        btn_row.append(&remove_btn);
        outer.append(&btn_row);

        // Remove button → collect checked packages and run removal
        let dialog_clone = dialog.clone();
        let window_clone = window.clone();
        let cbs = checkboxes.clone();
        remove_btn.connect_clicked(move |_| {
            let selected: Vec<String> = cbs
                .borrow()
                .iter()
                .filter(|(_, cb)| cb.is_active())
                .map(|(pkg, _)| pkg.clone())
                .collect();

            info!("Removing {} orphaned packages", selected.len());
            dialog_clone.close();

            if selected.is_empty() {
                return;
            }

            let mut args: Vec<&str> = vec!["-Rns", "--noconfirm"];
            let refs: Vec<&str> = selected.iter().map(|s| s.as_str()).collect();
            args.extend_from_slice(&refs);

            let commands = CommandSequence::new()
                .then(
                    Command::builder()
                        .aur()
                        .args(&args)
                        .description("Removing orphaned packages...")
                        .build(),
                )
                .build();

            task_runner::run(
                window_clone.upcast_ref(),
                commands,
                "Remove Orphaned Packages",
            );
        });

        toolbar.set_content(Some(&outer));
        dialog.set_content(Some(&toolbar));
        dialog.present();
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
    let btn_update_mirrorlist = extract_widget::<gtk4::Button>(page_builder, "btn_update_mirrorlist");
    let window = window.clone();
    btn_update_mirrorlist.connect_clicked(move |_| {
        info!("Servicing: Update Mirrorlist button clicked");

        let rate_mirrors_installed = core::is_package_installed("rate-mirrors");

        let mirror_mappings: Vec<(&str, &str, &str)> = vec![
            ("/etc/pacman.d/mirrorlist", "arch", "Arch"),
            ("/etc/pacman.d/chaotic-mirrorlist", "chaotic-aur", "Chaotic-AUR"),
            ("/etc/pacman.d/cachyos-mirrorlist", "cachyos", "CachyOS"),
            ("/etc/pacman.d/endeavouros-mirrorlist", "endeavouros", "EndeavourOS"),
            ("/etc/pacman.d/manjaro-mirrorlist", "manjaro", "Manjaro"),
            ("/etc/pacman.d/rebornos-mirrorlist", "rebornos", "RebornOS"),
            ("/etc/pacman.d/artix-mirrorlist", "artix", "Artix"),
        ];

        let mut commands = CommandSequence::new();

        if !rate_mirrors_installed {
            commands = commands.then(Command::builder()
                .aur()
                .args(&["-S", "--needed", "--noconfirm", "rate-mirrors"])
                .description("Installing rate-mirrors utility...")
                .build());
        }

        for (file_path, repo_id, repo_name) in mirror_mappings {
            if std::path::Path::new(file_path).exists() {
                let cmd = format!(
                    "rate-mirrors --allow-root --protocol https {} | tee {}",
                    repo_id, file_path
                );
                let description = format!("Updating {} mirrorlist...", repo_name);
                commands = commands.then(Command::builder()
                    .privileged()
                    .program("sh")
                    .args(&["-c", &cmd])
                    .description(&description)
                    .build());
            }
        }

        task_runner::run(window.upcast_ref(), commands.build(), "Update System Mirrorlists");
    });
}

fn setup_parallel_downloads(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_parallel_downloads = extract_widget::<gtk4::Button>(page_builder, "btn_parallel_downloads");
    let window = window.clone();
    btn_parallel_downloads.connect_clicked(move |_| {
        info!("Servicing: Change Parallel Downloads button clicked");
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

fn setup_xpackagemanager(page_builder: &Builder, window: &ApplicationWindow) {
    let btn_xpackagemanager = extract_widget::<gtk4::Button>(page_builder, "btn_xpackagemanager");
    let btn_xpackagemanager_uninstall = extract_widget::<gtk4::Button>(page_builder, "btn_xpackagemanager_uninstall");

    fn update_button_state(setup_btn: &gtk4::Button, uninstall_btn: &gtk4::Button, is_installed: bool) {
        if is_installed {
            setup_btn.set_label("Launch");
            setup_btn.add_css_class("suggested-action");
            uninstall_btn.set_visible(true);
        } else {
            setup_btn.set_label("Install");
            setup_btn.remove_css_class("suggested-action");
            uninstall_btn.set_visible(false);
        }
    }

    let is_installed = std::path::Path::new("/usr/bin/xpackagemanager").exists();
    update_button_state(&btn_xpackagemanager, &btn_xpackagemanager_uninstall, is_installed);

    let btn_setup_clone = btn_xpackagemanager.clone();
    let btn_uninstall_clone = btn_xpackagemanager_uninstall.clone();
    window.connect_is_active_notify(move |window| {
        if window.is_active() {
            let is_installed = std::path::Path::new("/usr/bin/xpackagemanager").exists();
            update_button_state(&btn_setup_clone, &btn_uninstall_clone, is_installed);
        }
    });

    let window_clone = window.clone();
    btn_xpackagemanager.connect_clicked(move |_| {
        info!("Servicing: xPackageManager button clicked");

        if std::path::Path::new("/usr/bin/xpackagemanager").exists() {
            info!("Launching xPackageManager...");
            if let Err(e) = std::process::Command::new("xpackagemanager")
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                log::error!("Failed to launch xPackageManager: {}", e);
            }
        } else {
            let commands = CommandSequence::new()
                .then(
                    Command::builder()
                        .privileged()
                        .program("pacman")
                        .args(&["-S", "--needed", "--noconfirm", "rust", "qt6-base", "qt6-declarative", "pacman", "flatpak", "git"])
                        .description("Installing build dependencies...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .normal()
                        .program("sh")
                        .args(&[
                            "-c",
                            "rm -rf /tmp/xpm-build && git clone https://github.com/MurderFromMars/xPackageManager.git /tmp/xpm-build",
                        ])
                        .description("Cloning xPackageManager source...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .normal()
                        .program("sh")
                        .args(&["-c", "cd /tmp/xpm-build && cargo build --release"])
                        .description("Building xPackageManager (this may take a few minutes)...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .privileged()
                        .program("sh")
                        .args(&[
                            "-c",
                            "mkdir -p /opt/xpackagemanager && install -Dm755 /tmp/xpm-build/target/release/xpackagemanager /opt/xpackagemanager/xpackagemanager && ln -sf /opt/xpackagemanager/xpackagemanager /usr/bin/xpackagemanager",
                        ])
                        .description("Installing binary to /opt/xpackagemanager...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .privileged()
                        .program("sh")
                        .args(&[
                            "-c",
                            r#"cat > /usr/share/applications/xpackagemanager.desktop << 'EOF'
[Desktop Entry]
Name=xPackage Manager
Comment=Modern package manager for Arch Linux
Exec=xpackagemanager
Icon=system-software-install
Terminal=false
Type=Application
Categories=System;PackageManager;
Keywords=package;manager;pacman;flatpak;
EOF"#,
                        ])
                        .description("Installing desktop entry...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .privileged()
                        .program("sh")
                        .args(&[
                            "-c",
                            r#"cat > /usr/share/mime/packages/x-alpm-package.xml << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="application/x-alpm-package">
    <comment>Arch Linux Package</comment>
    <glob pattern="*.pkg.tar.zst"/>
    <glob pattern="*.pkg.tar.xz"/>
    <glob pattern="*.pkg.tar.gz"/>
  </mime-type>
</mime-info>
EOF"#,
                        ])
                        .description("Installing MIME type definition...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .privileged()
                        .program("sh")
                        .args(&[
                            "-c",
                            r#"cat > /usr/share/polkit-1/actions/org.xpackagemanager.policy << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE policyconfig PUBLIC
 "-//freedesktop//DTD PolicyKit Policy Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/PolicyKit/1/policyconfig.dtd">
<policyconfig>
  <action id="org.xpackagemanager.pkexec">
    <description>Run xPackageManager privileged operations</description>
    <message>Authentication is required to manage packages</message>
    <defaults>
      <allow_any>auth_admin</allow_any>
      <allow_inactive>auth_admin</allow_inactive>
      <allow_active>auth_admin_keep</allow_active>
    </defaults>
    <annotate key="org.freedesktop.policykit.exec.path">/opt/xpackagemanager/xpackagemanager</annotate>
  </action>
</policyconfig>
EOF"#,
                        ])
                        .description("Installing polkit policy...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .privileged()
                        .program("sh")
                        .args(&[
                            "-c",
                            "update-desktop-database /usr/share/applications 2>/dev/null || true",
                        ])
                        .description("Updating desktop database...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .privileged()
                        .program("sh")
                        .args(&[
                            "-c",
                            "update-mime-database /usr/share/mime 2>/dev/null || true",
                        ])
                        .description("Updating MIME database...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .normal()
                        .program("rm")
                        .args(&["-rf", "/tmp/xpm-build"])
                        .description("Cleaning up temporary files...")
                        .build(),
                )
                .build();

            task_runner::run(
                window_clone.upcast_ref(),
                commands,
                "Install xPackageManager",
            );
        }
    });

    let window_clone = window.clone();
    btn_xpackagemanager_uninstall.connect_clicked(move |_| {
        info!("Servicing: xPackageManager uninstall button clicked");

        let commands = CommandSequence::new()
            .then(
                Command::builder()
                    .privileged()
                    .program("rm")
                    .args(&["-f", "/usr/bin/xpackagemanager"])
                    .description("Removing xPackageManager binary...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("rm")
                    .args(&["-rf", "/opt/xpackagemanager"])
                    .description("Removing application files...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("rm")
                    .args(&["-f", "/usr/share/applications/xpackagemanager.desktop"])
                    .description("Removing desktop entry...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("rm")
                    .args(&["-f", "/usr/share/mime/packages/x-alpm-package.xml"])
                    .description("Removing MIME type...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("rm")
                    .args(&["-f", "/usr/share/polkit-1/actions/org.xpackagemanager.policy"])
                    .description("Removing polkit policy...")
                    .build(),
            )
            .then(
                Command::builder()
                    .privileged()
                    .program("update-desktop-database")
                    .args(&["/usr/share/applications"])
                    .description("Updating desktop database...")
                    .build(),
            )
            .build();

        task_runner::run(
            window_clone.upcast_ref(),
            commands,
            "Uninstall xPackageManager",
        );
    });
}

/// Get the latest remote commit hash from the toolkit GitHub repository.
fn get_remote_commit() -> Option<String> {
    std::process::Command::new("git")
        .args(["ls-remote", config::links::TOOLKIT_REPO, "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .split_whitespace()
                .next()
                .map(|s| s.to_string())
        })
}

/// Get the locally stored commit hash from the last toolkit install/update.
fn get_local_commit() -> Option<String> {
    std::fs::read_to_string("/opt/xero-toolkit/.commit")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn setup_update_toolkit(page_builder: &Builder, window: &ApplicationWindow) {
    let btn = extract_widget::<gtk4::Button>(page_builder, "btn_update_toolkit");
    let window = window.clone();

    btn.connect_clicked(move |btn| {
        info!("Servicing: Update Toolkit button clicked");

        // Disable button while checking
        btn.set_sensitive(false);
        let btn_clone = btn.clone();

        // Check for updates
        let remote = get_remote_commit();
        let local = get_local_commit();

        btn_clone.set_sensitive(true);

        // If we can't reach GitHub, warn the user
        let remote_hash = match remote {
            Some(hash) => hash,
            None => {
                let dialog = adw::Window::new();
                dialog.set_title(Some("Xero Toolkit - Update"));
                dialog.set_default_size(420, 200);
                dialog.set_modal(true);
                dialog.set_transient_for(Some(&window));

                let toolbar = adw::ToolbarView::new();
                let header = adw::HeaderBar::new();
                toolbar.add_top_bar(&header);

                let content = GtkBox::new(Orientation::Vertical, 16);
                content.set_margin_top(24);
                content.set_margin_bottom(24);
                content.set_margin_start(24);
                content.set_margin_end(24);
                content.set_halign(gtk4::Align::Center);
                content.set_valign(gtk4::Align::Center);

                let icon = gtk4::Image::from_icon_name("network-error-symbolic");
                icon.set_pixel_size(48);
                content.append(&icon);

                let label = Label::new(Some(
                    "Could not reach GitHub to check for updates.\nPlease check your internet connection.",
                ));
                label.set_halign(gtk4::Align::Center);
                label.set_justify(gtk4::Justification::Center);
                content.append(&label);

                let ok_btn = gtk4::Button::with_label("OK");
                ok_btn.add_css_class("suggested-action");
                ok_btn.add_css_class("pill");
                ok_btn.set_halign(gtk4::Align::Center);
                let dialog_clone = dialog.clone();
                ok_btn.connect_clicked(move |_| dialog_clone.close());
                content.append(&ok_btn);

                toolbar.set_content(Some(&content));
                dialog.set_content(Some(&toolbar));
                dialog.present();
                return;
            }
        };

        // Check if already up to date
        let is_up_to_date = local
            .as_ref()
            .map(|l| l == &remote_hash)
            .unwrap_or(false);

        if is_up_to_date {
            let dialog = adw::Window::new();
            dialog.set_title(Some("Xero Toolkit - Update"));
            dialog.set_default_size(420, 200);
            dialog.set_modal(true);
            dialog.set_transient_for(Some(&window));

            let toolbar = adw::ToolbarView::new();
            let header = adw::HeaderBar::new();
            toolbar.add_top_bar(&header);

            let content = GtkBox::new(Orientation::Vertical, 16);
            content.set_margin_top(24);
            content.set_margin_bottom(24);
            content.set_margin_start(24);
            content.set_margin_end(24);
            content.set_halign(gtk4::Align::Center);
            content.set_valign(gtk4::Align::Center);

            let icon = gtk4::Image::from_icon_name("emblem-ok-symbolic");
            icon.set_pixel_size(48);
            content.append(&icon);

            let label = Label::new(Some("CyberXero Toolkit is already up to date!"));
            label.set_halign(gtk4::Align::Center);
            label.set_justify(gtk4::Justification::Center);
            content.append(&label);

            let hash_label = Label::new(Some(&format!("Commit: {}", &remote_hash[..12])));
            hash_label.add_css_class("dim-label");
            hash_label.add_css_class("caption");
            hash_label.set_halign(gtk4::Align::Center);
            content.append(&hash_label);

            let ok_btn = gtk4::Button::with_label("OK");
            ok_btn.add_css_class("suggested-action");
            ok_btn.add_css_class("pill");
            ok_btn.set_halign(gtk4::Align::Center);
            let dialog_clone = dialog.clone();
            ok_btn.connect_clicked(move |_| dialog_clone.close());
            content.append(&ok_btn);

            toolbar.set_content(Some(&content));
            dialog.set_content(Some(&toolbar));
            dialog.present();
            return;
        }

        // Updates available — show confirmation with commit info, then run update
        let dialog = adw::Window::new();
        dialog.set_title(Some("Xero Toolkit - Update Available"));
        dialog.set_default_size(480, 280);
        dialog.set_modal(true);
        dialog.set_transient_for(Some(&window));

        let toolbar = adw::ToolbarView::new();
        let header = adw::HeaderBar::new();
        toolbar.add_top_bar(&header);

        let content = GtkBox::new(Orientation::Vertical, 16);
        content.set_margin_top(24);
        content.set_margin_bottom(24);
        content.set_margin_start(24);
        content.set_margin_end(24);
        content.set_halign(gtk4::Align::Center);
        content.set_valign(gtk4::Align::Center);

        let icon = gtk4::Image::from_icon_name("software-update-available-symbolic");
        icon.set_pixel_size(48);
        content.append(&icon);

        let title_label = Label::new(Some("A new version is available!"));
        title_label.add_css_class("title-3");
        title_label.set_halign(gtk4::Align::Center);
        content.append(&title_label);

        let info_text = match &local {
            Some(l) => format!(
                "Current: {}\nLatest:  {}",
                &l[..l.len().min(12)],
                &remote_hash[..12]
            ),
            None => format!("Latest: {}", &remote_hash[..12]),
        };
        let info_label = Label::new(Some(&info_text));
        info_label.add_css_class("dim-label");
        info_label.add_css_class("monospace");
        info_label.set_halign(gtk4::Align::Center);
        content.append(&info_label);

        let note_label = Label::new(Some(
            "This will download, build, and install the latest version.\nThe toolkit will need to be restarted after updating.",
        ));
        note_label.set_wrap(true);
        note_label.set_halign(gtk4::Align::Center);
        note_label.set_justify(gtk4::Justification::Center);
        content.append(&note_label);

        let button_box = GtkBox::new(Orientation::Horizontal, 12);
        button_box.set_halign(gtk4::Align::Center);

        let cancel_btn = gtk4::Button::with_label("Cancel");
        cancel_btn.add_css_class("pill");
        cancel_btn.set_width_request(120);
        let dialog_cancel = dialog.clone();
        cancel_btn.connect_clicked(move |_| dialog_cancel.close());
        button_box.append(&cancel_btn);

        let update_btn = gtk4::Button::with_label("Update Now");
        update_btn.add_css_class("suggested-action");
        update_btn.add_css_class("pill");
        update_btn.set_width_request(120);

        let window_clone = window.clone();
        let dialog_update = dialog.clone();
        let remote_hash_clone = remote_hash.clone();
        update_btn.connect_clicked(move |_| {
            dialog_update.close();

            let repo_url = config::links::TOOLKIT_REPO;
            let commit_store_cmd = format!(
                "echo '{}' | sudo tee /opt/xero-toolkit/.commit > /dev/null",
                remote_hash_clone
            );

            let commands = CommandSequence::new()
                .then(
                    Command::builder()
                        .normal()
                        .program("sh")
                        .args(&[
                            "-c",
                            &format!(
                                "rm -rf /tmp/xero-toolkit-update && git clone --depth 1 {} /tmp/xero-toolkit-update",
                                repo_url
                            ),
                        ])
                        .description("Cloning latest CyberXero Toolkit from GitHub...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .normal()
                        .program("sh")
                        .args(&["-c", "cd /tmp/xero-toolkit-update && cargo build --release"])
                        .description("Building CyberXero Toolkit (this may take a few minutes)...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .privileged()
                        .program("sh")
                        .args(&[
                            "-c",
                            "install -Dm755 /tmp/xero-toolkit-update/target/release/xero-toolkit /opt/xero-toolkit/xero-toolkit",
                        ])
                        .description("Installing updated xero-toolkit binary...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .privileged()
                        .program("sh")
                        .args(&[
                            "-c",
                            "install -Dm755 /tmp/xero-toolkit-update/target/release/xero-authd /opt/xero-toolkit/xero-authd",
                        ])
                        .description("Installing updated xero-authd binary...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .privileged()
                        .program("sh")
                        .args(&[
                            "-c",
                            "install -Dm755 /tmp/xero-toolkit-update/target/release/xero-auth /opt/xero-toolkit/xero-auth",
                        ])
                        .description("Installing updated xero-auth binary...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .privileged()
                        .program("sh")
                        .args(&[
                            "-c",
                            "cp -f /tmp/xero-toolkit-update/sources/scripts/* /opt/xero-toolkit/sources/scripts/ && \
                             chmod 755 /opt/xero-toolkit/sources/scripts/* && \
                             cp -f /tmp/xero-toolkit-update/sources/systemd/* /opt/xero-toolkit/sources/systemd/",
                        ])
                        .description("Updating scripts and systemd units...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .privileged()
                        .program("sh")
                        .args(&[
                            "-c",
                            "if [ -d /tmp/xero-toolkit-update/extra-scripts/usr/local/bin ]; then \
                                cp -f /tmp/xero-toolkit-update/extra-scripts/usr/local/bin/* /usr/local/bin/ 2>/dev/null; \
                                chmod 755 /usr/local/bin/upd /usr/local/bin/grubup 2>/dev/null; \
                             fi; true",
                        ])
                        .description("Updating extra scripts...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .privileged()
                        .program("sh")
                        .args(&["-c", &commit_store_cmd])
                        .description("Recording update version...")
                        .build(),
                )
                .then(
                    Command::builder()
                        .normal()
                        .program("rm")
                        .args(&["-rf", "/tmp/xero-toolkit-update"])
                        .description("Cleaning up temporary files...")
                        .build(),
                )
                .build();

            task_runner::run(
                window_clone.upcast_ref(),
                commands,
                "Update CyberXero Toolkit",
            );
        });
        button_box.append(&update_btn);

        content.append(&button_box);
        toolbar.set_content(Some(&content));
        dialog.set_content(Some(&toolbar));
        dialog.present();
    });
}
