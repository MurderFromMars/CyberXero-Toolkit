//! Kernel Manager and SCX Scheduler page handlers.
//!
//! Handles:
//! - Linux kernel installation and removal
//! - Kernel headers management
//! - Kernel listing and status

use crate::ui::dialogs::warning::show_warning_confirmation;
use crate::ui::task_runner::{self, Command, CommandSequence};
use crate::ui::utils::extract_widget;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Box as GtkBox, Builder, Button, Image, Label, ListBox, Orientation};
use log::{info, warn};
use std::process::{Command as StdCommand, Stdio};
use std::sync::mpsc;

/// Set up all button handlers for the kernel manager page.
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    setup_kernel_lists(page_builder, window);
    setup_refresh_button(page_builder, window);
}

/// Initialize and populate kernel lists.
fn setup_kernel_lists(builder: &Builder, window: &ApplicationWindow) {
    let window = window.clone();
    let builder = builder.clone();
    glib::spawn_future_local(async move {
        scan_and_populate_kernels(&builder, &window).await;
    });
}

/// Set up refresh button to rescan kernels.
fn setup_refresh_button(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_refresh_kernels");
    let window = window.clone();
    let builder = builder.clone();

    button.connect_clicked(move |btn| {
        info!("Refresh kernels button clicked");
        let builder = builder.clone();
        let window = window.clone();

        // Make the button icon spin
        btn.set_sensitive(false);
        // Find the Image widget inside the Box child
        if let Some(box_child) = btn.child().and_downcast::<GtkBox>() {
            if let Some(image) = box_child.first_child().and_downcast::<Image>() {
                image.add_css_class("spinning");
            }
        }
        let btn_clone = btn.clone();

        glib::spawn_future_local(async move {
            scan_and_populate_kernels(&builder, &window).await;

            // Stop spinning and re-enable button after scan
            btn_clone.set_sensitive(true);
            // Find the Image widget inside the Box child
            if let Some(box_child) = btn_clone.child().and_downcast::<GtkBox>() {
                if let Some(image) = box_child.first_child().and_downcast::<Image>() {
                    image.remove_css_class("spinning");
                }
            }
        });
    });
}

/// Scan for available and installed kernels and populate lists.
async fn scan_and_populate_kernels(builder: &Builder, window: &ApplicationWindow) {
    info!("Scanning for kernels...");

    let builder = builder.clone();
    let window = window.clone();

    // Show loading state (keep the lists visible, just show loading indicator)
    let loading_box = extract_widget::<GtkBox>(&builder, "loading_box");
    loading_box.set_visible(true);

    // Create a channel to communicate between threads
    let (sender, receiver) = mpsc::channel();

    // Run blocking operations in a separate thread
    std::thread::spawn(move || {
        let available_result = get_available_kernels();
        let installed_result = get_installed_kernels();

        let available_kernels = match available_result {
            Ok(kernels) => kernels,
            Err(e) => {
                warn!("Failed to get available kernels: {}", e);
                Vec::new()
            }
        };

        let installed_kernels = match installed_result {
            Ok(kernels) => kernels,
            Err(e) => {
                warn!("Failed to get installed kernels: {}", e);
                Vec::new()
            }
        };

        info!(
            "Found {} available kernels, {} installed",
            available_kernels.len(),
            installed_kernels.len()
        );

        // Send results back to main thread
        let _ = sender.send((available_kernels, installed_kernels));
    });

    // Receive results in main thread and update UI
    glib::idle_add_local_once(move || {
        if let Ok((available_kernels, installed_kernels)) = receiver.recv() {
            populate_installed_list(&builder, &installed_kernels, &window);
            populate_available_list(&builder, &available_kernels, &installed_kernels, &window);
            update_status_labels(&builder, &available_kernels, &installed_kernels);

            // Hide loading state
            loading_box.set_visible(false);
        }
    });
}

/// Get list of available kernel packages from repositories.
/// This function searches for kernel headers and then derives the kernel package names.
/// Adapted from cachyos-kernel-manager logic.
fn get_available_kernels() -> anyhow::Result<Vec<String>> {
    // Get all packages in one call
    let output = StdCommand::new("pacman")
        .args(["-Sl"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("pacman -Sl failed"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    // First pass: collect all available packages
    let mut all_packages = std::collections::HashSet::new();
    let mut kernel_headers = Vec::new();

    for line in stdout.lines() {
        // Skip testing repo
        if line.contains("testing/") {
            continue;
        }

        // Parse lines like: core linux-headers 6.6.1-1
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let pkg_name = parts[1];

        // Collect all package names
        if pkg_name.starts_with("linux") {
            all_packages.insert(pkg_name.to_string());
        }

        // Find kernel headers (but not linux-api-headers)
        if pkg_name.starts_with("linux")
            && pkg_name.ends_with("-headers")
            && pkg_name != "linux-api-headers"
        {
            kernel_headers.push(pkg_name.to_string());
        }
    }

    // Second pass: for each headers package, check if kernel exists
    let mut kernels = Vec::new();
    for headers_pkg in kernel_headers {
        if let Some(kernel_name) = headers_pkg.strip_suffix("-headers") {
            // Check if the corresponding kernel package exists
            if all_packages.contains(kernel_name) {
                kernels.push(kernel_name.to_string());
            }
        }
    }

    kernels.sort();
    kernels.dedup();
    Ok(kernels)
}

/// Get list of installed kernel packages.
/// Only returns kernels that have both the kernel and headers installed.
fn get_installed_kernels() -> anyhow::Result<Vec<String>> {
    let output = StdCommand::new("pacman")
        .args(["-Q"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("pacman -Q failed"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut installed_headers = Vec::new();
    let mut all_packages = Vec::new();

    // First pass: collect all packages and identify headers
    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let pkg_name = line.split_whitespace().next().unwrap_or("");
        all_packages.push(pkg_name.to_string());

        // Find kernel headers
        if pkg_name.starts_with("linux")
            && pkg_name.ends_with("-headers")
            && pkg_name != "linux-api-headers"
        {
            installed_headers.push(pkg_name.to_string());
        }
    }

    let mut kernels = Vec::new();

    // Second pass: for each headers package, check if the kernel is also installed
    for headers_pkg in installed_headers {
        if let Some(kernel_name) = headers_pkg.strip_suffix("-headers") {
            // Check if the corresponding kernel package is installed
            if all_packages.contains(&kernel_name.to_string()) {
                kernels.push(kernel_name.to_string());
            }
        }
    }

    kernels.sort();
    kernels.dedup();
    Ok(kernels)
}

/// Populate the installed kernels list.
fn populate_installed_list(builder: &Builder, kernels: &[String], window: &ApplicationWindow) {
    let list = extract_widget::<ListBox>(builder, "installed_kernels_list");

    // Clear existing items
    while let Some(row) = list.first_child() {
        list.remove(&row);
    }

    // Add kernels with remove buttons
    for kernel in kernels {
        let row_box = GtkBox::new(Orientation::Horizontal, 8);
        row_box.set_margin_start(12);
        row_box.set_margin_end(12);
        row_box.set_margin_top(8);
        row_box.set_margin_bottom(8);

        let label = Label::new(Some(kernel));
        label.set_xalign(0.0);
        label.set_hexpand(true);
        row_box.append(&label);

        let remove_button = Button::new();
        remove_button.set_icon_name("trash-symbolic");
        remove_button.set_valign(gtk4::Align::Center);
        remove_button.add_css_class("flat");
        remove_button.add_css_class("destructive-action");

        let kernel_name = kernel.clone();
        let window_clone = window.clone();
        let builder_clone = builder.clone();
        remove_button.connect_clicked(move |_| {
            remove_kernel(&kernel_name, &window_clone, &builder_clone);
        });

        row_box.append(&remove_button);
        list.append(&row_box);
    }

    if kernels.is_empty() {
        let label = Label::new(Some("No kernels installed"));
        label.add_css_class("dim-label");
        label.set_margin_start(12);
        label.set_margin_end(12);
        label.set_margin_top(8);
        label.set_margin_bottom(8);
        list.append(&label);
    }
}

/// Populate the available kernels list (excluding installed ones).
fn populate_available_list(
    builder: &Builder,
    available: &[String],
    installed: &[String],
    window: &ApplicationWindow,
) {
    let list = extract_widget::<ListBox>(builder, "available_kernels_list");

    // Clear existing items
    while let Some(row) = list.first_child() {
        list.remove(&row);
    }

    // Add kernels that are not installed with install buttons
    let mut added = 0;
    for kernel in available {
        if !installed.contains(kernel) {
            let row_box = GtkBox::new(Orientation::Horizontal, 8);
            row_box.set_margin_start(12);
            row_box.set_margin_end(12);
            row_box.set_margin_top(8);
            row_box.set_margin_bottom(8);

            let label = Label::new(Some(kernel));
            label.set_xalign(0.0);
            label.set_hexpand(true);
            row_box.append(&label);

            let install_button = Button::new();
            install_button.set_icon_name("download-symbolic");
            install_button.set_valign(gtk4::Align::Center);
            install_button.add_css_class("flat");
            install_button.add_css_class("suggested-action");

            let kernel_name = kernel.clone();
            let window_clone = window.clone();
            let builder_clone = builder.clone();
            install_button.connect_clicked(move |_| {
                install_kernel(&kernel_name, &window_clone, &builder_clone);
            });

            row_box.append(&install_button);
            list.append(&row_box);
            added += 1;
        }
    }

    if added == 0 {
        let label = Label::new(Some("All available kernels are installed"));
        label.add_css_class("dim-label");
        label.set_margin_start(12);
        label.set_margin_end(12);
        label.set_margin_top(8);
        label.set_margin_bottom(8);
        list.append(&label);
    }
}

/// Update status labels with kernel counts.
fn update_status_labels(builder: &Builder, available: &[String], installed: &[String]) {
    let installed_count = extract_widget::<Label>(builder, "installed_count_label");
    let available_count = extract_widget::<Label>(builder, "available_count_label");

    installed_count.set_text(&format!("{} installed", installed.len()));

    let not_installed = available.iter().filter(|k| !installed.contains(k)).count();
    available_count.set_text(&format!("{} available", not_installed));
}

/// Install a kernel with its headers.
fn install_kernel(kernel_name: &str, window: &ApplicationWindow, builder: &Builder) {
    let headers = format!("{}-headers", kernel_name);
    let kernel_name = kernel_name.to_string();
    let window_clone = window.clone();
    let builder_clone = builder.clone();

    show_warning_confirmation(
        window.upcast_ref(),
        "Confirm Installation",
        &format!(
            "Install <b>{}</b> and <b>{}</b>?\n\n\
            This will download and install the kernel and its headers.",
            kernel_name, headers
        ),
        move || {
            info!("Installing {} and {}", kernel_name, headers);

            let commands = CommandSequence::new()
                .then(
                    Command::builder()
                        .aur()
                        .args(&["-S", "--noconfirm", "--needed", &kernel_name, &headers])
                        .description(&format!("Installing {} and {}...", kernel_name, headers))
                        .build(),
                )
                .build();

            // Run installation
            task_runner::run(window_clone.upcast_ref(), commands, "Install Kernel");

            // Schedule refresh after dialog closes
            glib::timeout_add_seconds_local(2, move || {
                if !task_runner::is_running() {
                    let builder = builder_clone.clone();
                    let window = window_clone.clone();
                    glib::spawn_future_local(async move {
                        scan_and_populate_kernels(&builder, &window).await;
                    });
                    glib::ControlFlow::Break
                } else {
                    glib::ControlFlow::Continue
                }
            });
        },
    );
}

/// Remove a kernel with its headers.
fn remove_kernel(kernel_name: &str, window: &ApplicationWindow, builder: &Builder) {
    let headers = format!("{}-headers", kernel_name);
    let kernel_name = kernel_name.to_string();
    let window_clone = window.clone();
    let builder_clone = builder.clone();

    show_warning_confirmation(
        window.upcast_ref(),
        "Confirm Removal",
        &format!(
            "Remove <b>{}</b> and <b>{}</b>?\n\n\
            <span foreground=\"red\" weight=\"bold\">Warning:</span> \
            This will uninstall the kernel and its headers.\n\
            Make sure you have at least one other kernel installed.",
            kernel_name, headers
        ),
        move || {
            info!("Removing {} and {}", kernel_name, headers);

            let commands = CommandSequence::new()
                .then(
                    Command::builder()
                        .aur()
                        .args(&["-R", "--noconfirm", &kernel_name, &headers])
                        .description(&format!("Removing {} and {}...", kernel_name, headers))
                        .build(),
                )
                .build();

            // Run removal
            task_runner::run(window_clone.upcast_ref(), commands, "Remove Kernel");

            // Schedule refresh after dialog closes
            glib::timeout_add_seconds_local(2, move || {
                if !task_runner::is_running() {
                    let builder = builder_clone.clone();
                    let window = window_clone.clone();
                    glib::spawn_future_local(async move {
                        scan_and_populate_kernels(&builder, &window).await;
                    });
                    glib::ControlFlow::Break
                } else {
                    glib::ControlFlow::Continue
                }
            });
        },
    );
}
