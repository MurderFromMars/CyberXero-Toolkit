//! Package and system utility functions.
//!
//! This module provides utilities for checking installed packages,
//! flatpaks, and system operations.

use super::aur;
use anyhow::Result;
use log::debug;

/// Check if a package is installed using AUR helper or pacman.
pub fn is_package_installed(package: &str) -> bool {
    debug!("Checking if package '{}' is installed", package);

    // Try AUR helper first
    if let Some(helper) = aur::detect() {
        if let Ok(output) = std::process::Command::new(helper)
            .args(["-Q", package])
            .output()
        {
            if output.status.success() {
                debug!("Package '{}' found via {}", package, helper);
                return true;
            }
        }
    }

    // Fallback to pacman
    let installed = std::process::Command::new("pacman")
        .args(["-Q", package])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if installed {
        debug!("Package '{}' found via pacman", package);
    } else {
        debug!("Package '{}' not installed", package);
    }

    installed
}

/// Check if a flatpak package is installed.
pub fn is_flatpak_installed(package: &str) -> bool {
    debug!("Checking if Flatpak '{}' is installed", package);

    // Use --columns=application to get only app IDs, one per line
    let installed = std::process::Command::new("flatpak")
        .args(["list", "--columns=application"])
        .output()
        .map(|output| {
            if output.status.success() {
                // Check for exact match on any line
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .any(|line| line.trim() == package)
            } else {
                false
            }
        })
        .unwrap_or(false);

    if installed {
        debug!("Flatpak '{}' found", package);
    } else {
        debug!("Flatpak '{}' not installed", package);
    }

    installed
}

/// Open a URL in the default browser.
pub fn open_url(url: &str) -> Result<()> {
    debug!("Opening URL: {}", url);
    std::process::Command::new("xdg-open").arg(url).spawn()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_package_installed_nonexistent() {
        // A package that definitely doesn't exist
        assert!(!is_package_installed(
            "this-package-definitely-does-not-exist-12345"
        ));
    }
}
