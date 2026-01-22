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
use gtk4::{ApplicationWindow, Builder, Button, Label, ListBox};
use log::{info, warn};
use std::process::Stdio;

/// Set up all button handlers for the kernel manager page.
pub fn setup_handlers(page_builder: &Builder, _main_builder: &Builder, window: &ApplicationWindow) {
    setup_kernel_lists(page_builder, window);
    setup_refresh_button(page_builder, window);
    setup_install_button(page_builder, window);
    setup_remove_button(page_builder, window);
}

/// Initialize and populate kernel lists.
fn setup_kernel_lists(builder: &Builder, window: &ApplicationWindow) {
    let window = window.clone();
    let builder = builder.clone();
    glib::MainContext::default().spawn_local(async move {
        scan_and_populate_kernels(&builder, &window).await;
    });
}

/// Set up refresh button to rescan kernels.
fn setup_refresh_button(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_refresh_kernels");
    let window = window.clone();
    let builder = builder.clone();

    button.connect_clicked(move |_| {
        info!("Refresh kernels button clicked");
        let builder = builder.clone();
        let window = window.clone();

        glib::MainContext::default().spawn_local(async move {
            scan_and_populate_kernels(&builder, &window).await;
        });
    });
}

/// Set up install button to install selected kernel.
fn setup_install_button(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_install_kernel");
    let window = window.clone();
    let builder = builder.clone();

    button.connect_clicked(move |_| {
        info!("Install kernel button clicked");

        let available_list = extract_widget::<ListBox>(&builder, "available_kernels_list");

        if let Some(row) = available_list.selected_row() {
            if let Some(label) = row.child().and_then(|w| w.downcast::<Label>().ok()) {
                let kernel_name = label.label().to_string();
                install_kernel(&kernel_name, &window, &builder);
            }
        } else {
            show_warning_confirmation(
                window.upcast_ref(),
                "No Selection",
                "Please select a kernel from the available list to install.",
                || {},
            );
        }
    });
}

/// Set up remove button to remove selected kernel.
fn setup_remove_button(builder: &Builder, window: &ApplicationWindow) {
    let button = extract_widget::<Button>(builder, "btn_remove_kernel");
    let window = window.clone();
    let builder = builder.clone();

    button.connect_clicked(move |_| {
        info!("Remove kernel button clicked");

        let installed_list = extract_widget::<ListBox>(&builder, "installed_kernels_list");

        if let Some(row) = installed_list.selected_row() {
            if let Some(label) = row.child().and_then(|w| w.downcast::<Label>().ok()) {
                let kernel_name = label.label().to_string();

                // Check if this is the running kernel
                if is_running_kernel(&kernel_name) {
                    show_warning_confirmation(
                        window.upcast_ref(),
                        "Cannot Remove Running Kernel",
                        &format!(
                            "<b>{}</b> is currently running!\n\n\
                            Please reboot into another kernel first.",
                            kernel_name
                        ),
                        || {},
                    );
                    return;
                }

                remove_kernel(&kernel_name, &window, &builder);
            }
        } else {
            show_warning_confirmation(
                window.upcast_ref(),
                "No Selection",
                "Please select a kernel from the installed list to remove.",
                || {},
            );
        }
    });
}

/// Scan for available and installed kernels and populate lists.
async fn scan_and_populate_kernels(builder: &Builder, _window: &ApplicationWindow) {
    info!("Scanning for kernels...");

    // Get available kernels
    let available_kernels = match get_available_kernels().await {
        Ok(kernels) => kernels,
        Err(e) => {
            warn!("Failed to get available kernels: {}", e);
            Vec::new()
        }
    };

    // Get installed kernels
    let installed_kernels = match get_installed_kernels().await {
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

    // Update UI
    populate_installed_list(builder, &installed_kernels);
    populate_available_list(builder, &available_kernels, &installed_kernels);
    update_status_labels(builder, &available_kernels, &installed_kernels);
}

/// Get list of available kernel packages from repositories.
async fn get_available_kernels() -> anyhow::Result<Vec<String>> {
    let output = tokio::process::Command::new("pacman")
        .args(["-Ss", "^linux"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("pacman -Ss failed"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut kernels = Vec::new();

    for line in stdout.lines() {
        // Skip empty lines and description lines (start with spaces)
        if line.trim().is_empty() || line.starts_with(' ') {
            continue;
        }

        // Parse lines like: extra/linux 6.6.1-1
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Some(pkg_name) = parts[0].split('/').next_back() {
                // Filter: must start with 'linux' and be a kernel package
                if pkg_name.starts_with("linux")
                    && !pkg_name.contains("firmware")
                    && !pkg_name.contains("docs")
                    && !pkg_name.contains("api-headers")
                    && !pkg_name.contains("tools")
                    && !pkg_name.contains("meta")
                    && !pkg_name.ends_with("-headers")
                {
                    // Only add actual kernel packages
                    if pkg_name == "linux" || pkg_name.contains('-') {
                        kernels.push(pkg_name.to_string());
                    }
                }
            }
        }
    }

    kernels.sort();
    kernels.dedup();
    Ok(kernels)
}

/// Get list of installed kernel packages.
async fn get_installed_kernels() -> anyhow::Result<Vec<String>> {
    let output = tokio::process::Command::new("pacman")
        .args(["-Q"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("pacman -Q failed"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut kernels = Vec::new();

    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let pkg_name = line.split_whitespace().next().unwrap_or("");

        // Filter: must start with 'linux' and be a kernel package
        if pkg_name.starts_with("linux")
            && !pkg_name.contains("firmware")
            && !pkg_name.contains("docs")
            && !pkg_name.contains("api-headers")
            && !pkg_name.contains("tools")
            && !pkg_name.contains("meta")
            && !pkg_name.ends_with("-headers")
        {
            // Only add actual kernel packages
            if pkg_name == "linux" || pkg_name.contains('-') {
                kernels.push(pkg_name.to_string());
            }
        }
    }

    kernels.sort();
    kernels.dedup();
    Ok(kernels)
}

/// Check if a kernel is currently running.
fn is_running_kernel(kernel_name: &str) -> bool {
    if let Ok(output) = std::process::Command::new("uname").arg("-r").output() {
        let running = String::from_utf8_lossy(&output.stdout);
        return running.contains(kernel_name);
    }
    false
}

/// Populate the installed kernels list.
fn populate_installed_list(builder: &Builder, kernels: &[String]) {
    let list = extract_widget::<ListBox>(builder, "installed_kernels_list");

    // Clear existing items
    while let Some(row) = list.first_child() {
        list.remove(&row);
    }

    // Add kernels
    for kernel in kernels {
        let label = Label::new(Some(kernel));
        label.set_xalign(0.0);
        label.set_margin_start(12);
        label.set_margin_end(12);
        label.set_margin_top(8);
        label.set_margin_bottom(8);

        // Highlight running kernel
        if is_running_kernel(kernel) {
            label.add_css_class("accent");
            label.set_markup(&format!("<b>{}</b> (Running)", kernel));
        }

        list.append(&label);
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
fn populate_available_list(builder: &Builder, available: &[String], installed: &[String]) {
    let list = extract_widget::<ListBox>(builder, "available_kernels_list");

    // Clear existing items
    while let Some(row) = list.first_child() {
        list.remove(&row);
    }

    // Add kernels that are not installed
    let mut added = 0;
    for kernel in available {
        if !installed.contains(kernel) {
            let label = Label::new(Some(kernel));
            label.set_xalign(0.0);
            label.set_margin_start(12);
            label.set_margin_end(12);
            label.set_margin_top(8);
            label.set_margin_bottom(8);

            list.append(&label);
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
            glib::timeout_add_seconds_local(1, move || {
                if !task_runner::is_running() {
                    let builder = builder_clone.clone();
                    let window = window_clone.clone();
                    glib::MainContext::default().spawn_local(async move {
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
            glib::timeout_add_seconds_local(1, move || {
                if !task_runner::is_running() {
                    let builder = builder_clone.clone();
                    let window = window_clone.clone();
                    glib::MainContext::default().spawn_local(async move {
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
