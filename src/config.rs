//! Centralized configuration and constants for the application.

/// Application information constants.
pub mod app_info {
    pub const NAME: &str = "xero-toolkit";
    pub const ID: &str = "xyz.xerolinux.xero-toolkit";
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
}
