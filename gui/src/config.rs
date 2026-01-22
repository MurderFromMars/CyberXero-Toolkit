//! Centralized configuration and constants for the application.

/// Application information constants.
pub mod app_info {
    pub const NAME: &str = "xero-toolkit";
    pub const ID: &str = "xyz.xerolinux.xero-toolkit";
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
}

/// Sidebar configuration.
pub mod sidebar {
    pub const MIN_WIDTH: i32 = 200;
    pub const MAX_WIDTH: i32 = 400;
}

/// External links.
pub mod links {
    pub const DISCORD: &str = "https://discord.xerolinux.xyz/";
    pub const YOUTUBE: &str = "https://www.youtube.com/@XeroLinux";
    pub const WEBSITE: &str = "https://xerolinux.xyz/";
    pub const DONATE: &str = "https://ko-fi.com/xerolinux";
}

/// Binary paths for system executables.
pub mod paths {
    use std::path::PathBuf;

    /// Path to the xero-authd daemon binary.
    pub const DAEMON: &str = "/opt/xero-toolkit/xero-authd";

    /// Path to the xero-auth client binary.
    pub const CLIENT: &str = "/opt/xero-toolkit/xero-auth";

    /// Path to the scripts directory.
    pub const SCRIPTS: &str = "/opt/xero-toolkit/scripts";

    /// Path to the desktop file in system applications.
    pub const DESKTOP_FILE: &str = "/usr/share/applications/xero-toolkit.desktop";

    /// Path to the system-wide autostart desktop file.
    pub const SYSTEM_AUTOSTART: &str = "/etc/xdg/autostart/xero-toolkit.desktop";

    /// Get the daemon path as a PathBuf.
    pub fn daemon() -> PathBuf {
        PathBuf::from(DAEMON)
    }

    /// Get the client path as a PathBuf.
    pub fn client() -> PathBuf {
        PathBuf::from(CLIENT)
    }

    /// Get the scripts path as a PathBuf.
    pub fn scripts() -> PathBuf {
        PathBuf::from(SCRIPTS)
    }

    /// Get the desktop file path as a PathBuf.
    pub fn desktop_file() -> PathBuf {
        PathBuf::from(DESKTOP_FILE)
    }

    /// Get the system autostart path as a PathBuf.
    pub fn system_autostart() -> PathBuf {
        PathBuf::from(SYSTEM_AUTOSTART)
    }
}

/// Cached environment variables read at startup.
pub mod env {
    use std::sync::OnceLock;

    static ENV: OnceLock<Env> = OnceLock::new();

    /// Cached environment variables.
    pub struct Env {
        pub user: String,
        pub home: String,
    }

    impl Env {
        fn new() -> anyhow::Result<Self> {
            Ok(Self {
                user: std::env::var("USER")
                    .map_err(|_| anyhow::anyhow!("USER environment variable is not set"))?,
                home: std::env::var("HOME")
                    .map_err(|_| anyhow::anyhow!("HOME environment variable is not set"))?,
            })
        }
    }

    /// Initialize environment variables. Must be called at application startup.
    /// Returns an error if required environment variables (USER, HOME) are not set.
    pub fn init() -> anyhow::Result<()> {
        ENV.set(Env::new()?)
            .map_err(|_| anyhow::anyhow!("Environment variables already initialized"))?;
        Ok(())
    }

    /// Get the cached environment variables.
    /// Panics if not initialized (call `init()` at application startup).
    pub fn get() -> &'static Env {
        ENV.get()
            .expect("Environment variables not initialized. Call config::env::init() at startup.")
    }
}

/// Debug environment variables for seasonal effects.
pub mod seasonal_debug {
    pub const ENABLE_SNOW: &str = "XERO_TOOLKIT_ENABLE_SNOW";
    pub const ENABLE_HALLOWEEN: &str = "XERO_TOOLKIT_ENABLE_HALLOWEEN";

    /// Check if an environment variable is set to enable an effect.
    /// Returns `Some(true)` if enabled, `Some(false)` if explicitly disabled, `None` if not set.
    pub fn check_effect_env(var_name: &str) -> Option<bool> {
        std::env::var(var_name).ok().and_then(|value| {
            // Try parsing as boolean first
            if let Ok(enabled) = value.parse::<bool>() {
                return Some(enabled);
            }

            // Check for truthy/falsy string values (case-insensitive)
            let lower = value.to_lowercase();
            match lower.as_str() {
                "1" | "true" | "yes" => Some(true),
                "0" | "false" | "no" => Some(false),
                _ => None,
            }
        })
    }
}

/// UI resource paths for GResource files.
pub mod resources {
    /// Main application window UI.
    pub const MAIN_UI: &str = "/xyz/xerolinux/xero-toolkit/ui/main.ui";

    /// Icons resource path.
    pub const ICONS: &str = "/xyz/xerolinux/xero-toolkit/icons";

    /// CSS stylesheet resource path.
    pub const CSS: &str = "/xyz/xerolinux/xero-toolkit/css/style.css";

    /// Dialog UI resources.
    pub mod dialogs {
        pub const ABOUT: &str = "/xyz/xerolinux/xero-toolkit/ui/dialogs/about_dialog.ui";
        pub const DEPENDENCY_ERROR: &str =
            "/xyz/xerolinux/xero-toolkit/ui/dialogs/dependency_error_dialog.ui";
        pub const DOWNLOAD: &str = "/xyz/xerolinux/xero-toolkit/ui/dialogs/download_dialog.ui";
        pub const DOWNLOAD_SETUP: &str =
            "/xyz/xerolinux/xero-toolkit/ui/dialogs/download_setup_dialog.ui";
        pub const SELECTION: &str = "/xyz/xerolinux/xero-toolkit/ui/dialogs/selection_dialog.ui";
        pub const TASK_LIST: &str = "/xyz/xerolinux/xero-toolkit/ui/dialogs/task_list_dialog.ui";
        pub const TERMINAL: &str = "/xyz/xerolinux/xero-toolkit/ui/dialogs/terminal_dialog.ui";
        pub const WARNING: &str = "/xyz/xerolinux/xero-toolkit/ui/dialogs/warning_dialog.ui";
        pub const XEROLINUX_CHECK: &str =
            "/xyz/xerolinux/xero-toolkit/ui/dialogs/xerolinux_check_dialog.ui";
    }

    /// Page/tab UI resources.
    pub mod tabs {
        pub const CONTAINERS_VMS: &str = "/xyz/xerolinux/xero-toolkit/ui/tabs/containers_vms.ui";
        pub const CUSTOMIZATION: &str = "/xyz/xerolinux/xero-toolkit/ui/tabs/customization.ui";
        pub const DRIVERS: &str = "/xyz/xerolinux/xero-toolkit/ui/tabs/drivers.ui";
        pub const GAMESCOPE: &str = "/xyz/xerolinux/xero-toolkit/ui/tabs/gamescope.ui";
        pub const GAMING_TOOLS: &str = "/xyz/xerolinux/xero-toolkit/ui/tabs/gaming_tools.ui";
        pub const KERNEL_MANAGER_SCX: &str =
            "/xyz/xerolinux/xero-toolkit/ui/tabs/kernel_manager_scx.ui";
        pub const MAIN_PAGE: &str = "/xyz/xerolinux/xero-toolkit/ui/tabs/main_page.ui";
        pub const MULTIMEDIA_TOOLS: &str =
            "/xyz/xerolinux/xero-toolkit/ui/tabs/multimedia_tools.ui";
        pub const SERVICING_SYSTEM_TWEAKS: &str =
            "/xyz/xerolinux/xero-toolkit/ui/tabs/servicing_system_tweaks.ui";
    }
}
