//! Autostart management for the toolkit.
//!
//! Handles enabling/disabling autostart by managing the desktop file
//! in the user's autostart directory.

use crate::config;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::process::Command;

/// Get the autostart desktop file path
pub fn get_autostart_path() -> PathBuf {
    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
    config_dir.join("autostart").join("cyberxero-toolkit.desktop")
}

/// Check if autostart is enabled
pub fn is_enabled() -> bool {
    get_autostart_path().exists() || config::paths::system_autostart().exists()
}

/// Enable autostart by creating a symlink to the desktop file in autostart directory
pub fn enable() -> Result<(), std::io::Error> {
    let autostart_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("autostart");

    // Create autostart directory if it doesn't exist
    fs::create_dir_all(&autostart_dir)?;

    let target = get_autostart_path();

    // Remove existing file/symlink if present
    if target.symlink_metadata().is_ok() {
        fs::remove_file(&target)?;
    }

    let source = config::paths::desktop_file();
    if source.exists() {
        symlink(source, target)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Desktop file not found in system applications",
        ))
    }
}

/// Disable autostart by removing the desktop file
pub fn disable() -> Result<(), std::io::Error> {
    let path = get_autostart_path();
    if path.symlink_metadata().is_ok() {
        fs::remove_file(path)?;
    }

    let system_path = config::paths::system_autostart();
    if system_path.exists() {
        // Use pkexec to remove the file with root privileges
        let status = Command::new("pkexec").arg("rm").arg(system_path).status()?;

        if !status.success() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Failed to remove system-wide autostart file",
            ));
        }
    }
    Ok(())
}
