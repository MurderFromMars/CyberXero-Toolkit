//! UI utility functions for widget extraction and common operations.

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::Builder;
use std::process::Command;

/// Helper to extract widgets from builder with consistent error handling.
pub fn extract_widget<T: IsA<glib::Object>>(builder: &Builder, name: &str) -> T {
    builder
        .object(name)
        .unwrap_or_else(|| panic!("Failed to get widget with id '{}'", name))
}

/// Run a command and return stdout as a trimmed string.
pub fn run_command(program: &str, args: &[&str]) -> Option<String> {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

/// Check if a systemd service is enabled.
pub fn is_service_enabled(service: &str) -> bool {
    run_command("systemctl", &["is-enabled", service])
        .map(|s| s.to_lowercase().contains("enabled"))
        .unwrap_or(false)
}

/// Check if a systemd user service is enabled.
pub fn is_user_service_enabled(service: &str) -> bool {
    run_command("systemctl", &["--user", "is-enabled", service])
        .map(|s| s.to_lowercase().contains("enabled"))
        .unwrap_or(false)
}

/// Check if a pacman package is installed.
pub fn is_package_installed(package: &str) -> bool {
    Command::new("pacman")
        .args(["-Q", package])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Check if a path exists.
pub fn path_exists(path: &str) -> bool {
    std::path::Path::new(path).exists()
}
