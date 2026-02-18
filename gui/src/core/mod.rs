//! Core functionality and business logic.
//!
//! This module contains:
//! - `aur`: AUR helper detection and management
//! - `daemon`: Daemon management for xero-auth
//! - `download`: File download functionality
//! - `package`: Package and flatpak checking utilities
//! - `system_check`: System dependency and distribution validation

pub mod aur;
pub mod autostart;
pub mod daemon;
pub mod download;
pub mod package;
pub mod system_check;

// Re-export commonly used items
pub use aur::get as aur_helper;
pub use package::{is_flatpak_installed, is_package_installed, is_package_in_repos};
