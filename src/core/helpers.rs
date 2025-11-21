//! Helper functions for package and system checks.

use crate::utils;

/// Check if a package is installed using AUR helper (or fallback to pacman)
pub fn is_package_installed(package: &str) -> bool {
    if let Some(helper) = utils::detect_aur_helper() {
        if let Ok(output) = std::process::Command::new(helper)
            .args(&["-Q", package])
            .output()
        {
            if output.status.success() {
                return true;
            }
        }
    }

    std::process::Command::new("pacman")
        .args(&["-Q", package])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Check if a flatpak package is installed (apps, runtimes, and extensions)
pub fn is_flatpak_installed(package: &str) -> bool {
    std::process::Command::new("flatpak")
        .args(&["list"])
        .output()
        .map(|output| {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).contains(package)
            } else {
                false
            }
        })
        .unwrap_or(false)
}
