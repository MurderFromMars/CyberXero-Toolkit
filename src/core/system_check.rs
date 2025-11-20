//! System dependency checks and validation.

use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Builder, Button, Label};
use log::{error, info, warn};

/// Result of dependency check containing missing dependencies.
#[derive(Debug, Clone)]
pub struct DependencyCheckResult {
    pub flatpak_missing: bool,
    pub aur_helper_missing: bool,
}

impl DependencyCheckResult {
    /// Check if any dependencies are missing.
    pub fn has_missing_dependencies(&self) -> bool {
        self.flatpak_missing || self.aur_helper_missing
    }

    /// Get list of missing dependency names.
    pub fn missing_dependencies(&self) -> Vec<&str> {
        let mut missing = Vec::new();
        if self.flatpak_missing {
            missing.push("flatpak");
        }
        if self.aur_helper_missing {
            missing.push("paru or yay");
        }
        missing
    }

    /// Generate formatted list of missing dependencies for display.
    pub fn format_missing_list(&self) -> String {
        self.missing_dependencies()
            .iter()
            .map(|dep| format!("• <b>{}</b>", dep))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Generate installation hint based on missing dependencies.
    pub fn generate_install_hint(&self) -> String {
        let mut hints = Vec::new();

        if self.flatpak_missing {
            hints.push("Install flatpak: <tt>sudo pacman -S flatpak</tt>");
        }
        if self.aur_helper_missing {
            hints.push("AUR Helper repositories:\n• Paru: <a href=\"https://github.com/Morganamilo/paru\">https://github.com/Morganamilo/paru</a>\n• Yay: <a href=\"https://github.com/Jguer/yay\">https://github.com/Jguer/yay</a>");
        }

        if hints.is_empty() {
            return String::new();
        }

        hints.join("\n\n")
    }
}

/// Check if flatpak is installed and available.
fn check_flatpak() -> bool {
    info!("Checking for flatpak availability");
    match std::process::Command::new("flatpak")
        .arg("--version")
        .output()
    {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            info!("flatpak found: {}", version.trim());
            true
        }
        Ok(_) => {
            warn!("flatpak command exists but returned error");
            false
        }
        Err(_) => {
            warn!("flatpak not found in PATH");
            false
        }
    }
}

/// Check if an AUR helper (paru or yay) is installed.
fn check_aur_helper() -> bool {
    info!("Checking for AUR helper availability");

    if let Ok(output) = std::process::Command::new("paru").arg("--version").output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            info!("paru found: {}", version.trim());
            return true;
        }
    }

    if let Ok(output) = std::process::Command::new("yay").arg("--version").output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            info!("yay found: {}", version.trim());
            return true;
        }
    }

    warn!("No AUR helper (paru or yay) found in PATH");
    false
}

/// Check if current distribution is XeroLinux.
fn is_xerolinux() -> bool {
    get_distribution_name()
        .map(|name| name.to_lowercase().contains("xerolinux"))
        .unwrap_or(false)
}

/// Get distribution name from os-release files.
fn get_distribution_name() -> Option<String> {
    use std::fs;

    // Try /etc/os-release first (most common)
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        if let Some(name) = parse_os_release_name(&content) {
            return Some(name);
        }
    }

    // Fallback to /usr/lib/os-release
    if let Ok(content) = fs::read_to_string("/usr/lib/os-release") {
        if let Some(name) = parse_os_release_name(&content) {
            return Some(name);
        }
    }

    // Fallback to /etc/lsb-release
    if let Ok(content) = fs::read_to_string("/etc/lsb-release") {
        for line in content.lines() {
            if line.starts_with("DISTRIB_ID=") {
                let name = line
                    .trim_start_matches("DISTRIB_ID=")
                    .trim_matches('"')
                    .to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
    }

    None
}

/// Parse NAME field from os-release content.
fn parse_os_release_name(content: &str) -> Option<String> {
    // Try NAME first
    for line in content.lines() {
        if line.starts_with("NAME=") {
            let name = line
                .trim_start_matches("NAME=")
                .trim_matches('"')
                .to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }

    // Fallback to PRETTY_NAME
    for line in content.lines() {
        if line.starts_with("PRETTY_NAME=") {
            let name = line
                .trim_start_matches("PRETTY_NAME=")
                .trim_matches('"')
                .to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }

    // Final fallback to ID
    for line in content.lines() {
        if line.starts_with("ID=") {
            let name = line
                .trim_start_matches("ID=")
                .trim_matches('"')
                .to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }

    None
}

/// Perform all dependency checks and return results.
pub fn check_dependencies() -> DependencyCheckResult {
    info!("Performing system dependency checks");

    let flatpak_missing = !check_flatpak();
    let aur_helper_missing = !check_aur_helper();

    let result = DependencyCheckResult {
        flatpak_missing,
        aur_helper_missing,
    };

    if result.has_missing_dependencies() {
        let issues = result.missing_dependencies();
        error!("Issues detected: {}", issues.join(", "));
    } else {
        info!("All required dependencies are available");
    }

    result
}

/// Check if running on XeroLinux distribution.
pub fn check_xerolinux_distribution() -> bool {
    info!("Checking Linux distribution compatibility");
    let is_xero = is_xerolinux();

    if !is_xero {
        let distro_name = get_distribution_name().unwrap_or_else(|| "Unknown".to_string());
        warn!("Current distribution: {}", distro_name);
        warn!("This application is designed specifically for XeroLinux");
    } else {
        info!("XeroLinux detected - proceeding with application startup");
    }

    is_xero
}

/// Show XeroLinux distribution error dialog.
pub fn show_xerolinux_error_dialog(main_window: &ApplicationWindow) {
    error!("Showing XeroLinux distribution error dialog");

    let distro_name = get_distribution_name().unwrap_or_else(|| "Unknown".to_string());

    // Load error dialog from UI file
    let builder =
        Builder::from_resource("/xyz/xerolinux/xero-toolkit/ui/xerolinux_check_dialog.ui");

    let error_window: gtk4::Window = builder
        .object("xerolinux_error_window")
        .expect("Failed to get xerolinux_error_window");

    let distro_label: Label = builder
        .object("distro_label")
        .expect("Failed to get distro_label");

    let exit_button: Button = builder
        .object("exit_button")
        .expect("Failed to get exit_button");

    distro_label.set_label(&format!("Current distribution: <b>{}</b>", distro_name));

    error_window.set_transient_for(Some(main_window));

    let main_window_clone = main_window.clone();
    exit_button.connect_clicked(move |_| {
        error!("User clicked exit on XeroLinux error dialog");
        main_window_clone.close();
        std::process::exit(1);
    });

    error_window.show();
}

/// Show dependency error dialog and prevent app from continuing.
pub fn show_dependency_error_dialog(
    main_window: &ApplicationWindow,
    check_result: &DependencyCheckResult,
) {
    error!("Showing dependency error dialog");

    // Load error dialog from UI file
    let builder =
        Builder::from_resource("/xyz/xerolinux/xero-toolkit/ui/dependency_error_dialog.ui");

    let error_window: gtk4::Window = builder
        .object("dependency_error_window")
        .expect("Failed to get dependency_error_window");

    let missing_deps_label: Label = builder
        .object("missing_deps_label")
        .expect("Failed to get missing_deps_label");

    let install_hint_label: Label = builder
        .object("install_hint_label")
        .expect("Failed to get install_hint_label");

    let exit_button: Button = builder
        .object("exit_button")
        .expect("Failed to get exit_button");

    missing_deps_label.set_label(&check_result.format_missing_list());

    install_hint_label.set_label(&check_result.generate_install_hint());

    error_window.set_transient_for(Some(main_window));

    let main_window_clone = main_window.clone();
    exit_button.connect_clicked(move |_| {
        error!("User clicked exit on dependency error dialog");
        main_window_clone.close();
        std::process::exit(1);
    });

    error_window.show();
}

/// Check system requirements (XeroLinux distribution and dependencies) on app startup.
/// Shows appropriate error dialog if checks fail.
/// Returns true if all checks pass, false otherwise.
pub fn check_system_requirements(main_window: &ApplicationWindow) -> bool {
    // First check: XeroLinux distribution
    if !check_xerolinux_distribution() {
        error!("Cannot start application - not running on XeroLinux");
        show_xerolinux_error_dialog(main_window);
        return false;
    }

    // Second check: Required dependencies
    let check_result = check_dependencies();
    if check_result.has_missing_dependencies() {
        error!("Cannot start application - missing required dependencies");
        show_dependency_error_dialog(main_window, &check_result);
        return false;
    }

    info!("All checks passed successfully");
    true
}

