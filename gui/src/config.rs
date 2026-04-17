//! Centralized configuration and constants for the application.

/// Application information constants.
pub mod app_info {
    pub const NAME: &str = "cyberxero-toolkit";
    pub const ID: &str = "xyz.cyberxero.cyberxero-toolkit";
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");
}

/// Sidebar configuration.
pub mod sidebar {
    pub const MIN_WIDTH: i32 = 160;
    pub const MAX_WIDTH: i32 = 400;
}

/// External links.
pub mod links {
    pub const DISCORD: &str = "https://discord.gg/2pdhYusbKV";
    pub const YOUTUBE: &str = "https://www.youtube.com/@MurderFromMars";
    pub const GITHUB: &str = "https://github.com/MurderFromMars";
    pub const TOOLKIT_REPO: &str = "https://github.com/MurderFromMars/CyberXero-Toolkit.git";
}

/// Binary paths for system executables.
pub mod paths {
    use std::path::PathBuf;

    /// Path to the cyberxero-authd daemon binary.
    pub const DAEMON: &str = "/opt/cyberxero-toolkit/cyberxero-authd";

    /// Path to the cyberxero-auth client binary.
    pub const CLIENT: &str = "/opt/cyberxero-toolkit/cyberxero-auth";

    /// Path to the sources directory (contains scripts and systemd).
    #[allow(dead_code)]
    pub const SOURCES: &str = "/opt/cyberxero-toolkit/sources";

    /// Path to the scripts directory.
    pub const SCRIPTS: &str = "/opt/cyberxero-toolkit/sources/scripts";

    /// Path to the systemd units directory.
    pub const SYSTEMD: &str = "/opt/cyberxero-toolkit/sources/systemd";

    /// Path to the desktop file in system applications.
    pub const DESKTOP_FILE: &str = "/usr/share/applications/cyberxero-toolkit.desktop";

    /// Path to the system-wide autostart desktop file.
    pub const SYSTEM_AUTOSTART: &str = "/etc/xdg/autostart/cyberxero-toolkit.desktop";

    /// Get the daemon path as a PathBuf.
    pub fn daemon() -> PathBuf {
        PathBuf::from(DAEMON)
    }

    /// Get the client path as a PathBuf.
    pub fn client() -> PathBuf {
        PathBuf::from(CLIENT)
    }

    /// Get the sources path as a PathBuf.
    #[allow(dead_code)]
    pub fn sources() -> PathBuf {
        PathBuf::from(SOURCES)
    }

    /// Get the scripts path as a PathBuf.
    pub fn scripts() -> PathBuf {
        PathBuf::from(SCRIPTS)
    }

    /// Get the systemd units path as a PathBuf.
    pub fn systemd() -> PathBuf {
        PathBuf::from(SYSTEMD)
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
    pub const ENABLE_SNOW: &str = "CYBERXERO_TOOLKIT_ENABLE_SNOW";
    pub const ENABLE_HALLOWEEN: &str = "CYBERXERO_TOOLKIT_ENABLE_HALLOWEEN";

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
    pub const MAIN_UI: &str = "/xyz/cyberxero/cyberxero-toolkit/ui/main.ui";

    /// Icons resource path.
    pub const ICONS: &str = "/xyz/cyberxero/cyberxero-toolkit/icons";

    /// CSS stylesheet resource path.
    pub const CSS: &str = "/xyz/cyberxero/cyberxero-toolkit/css/style.css";

    /// Dialog UI resources.
    pub mod dialogs {
        pub const ABOUT: &str = "/xyz/cyberxero/cyberxero-toolkit/ui/dialogs/about_dialog.ui";
        pub const DEPENDENCY_ERROR: &str =
            "/xyz/cyberxero/cyberxero-toolkit/ui/dialogs/dependency_error_dialog.ui";
        pub const DOWNLOAD: &str = "/xyz/cyberxero/cyberxero-toolkit/ui/dialogs/download_dialog.ui";
        pub const DOWNLOAD_SETUP: &str =
            "/xyz/cyberxero/cyberxero-toolkit/ui/dialogs/download_setup_dialog.ui";
        pub const SCHEDULER_SELECTION: &str =
            "/xyz/cyberxero/cyberxero-toolkit/ui/dialogs/scheduler_selection_dialog.ui";
        pub const SELECTION: &str = "/xyz/cyberxero/cyberxero-toolkit/ui/dialogs/selection_dialog.ui";
        pub const TASK_LIST: &str = "/xyz/cyberxero/cyberxero-toolkit/ui/dialogs/task_list_dialog.ui";
        pub const TERMINAL: &str = "/xyz/cyberxero/cyberxero-toolkit/ui/dialogs/terminal_dialog.ui";
        pub const WARNING: &str = "/xyz/cyberxero/cyberxero-toolkit/ui/dialogs/warning_dialog.ui";
    }

    /// Page/tab UI resources.
    pub mod tabs {
        pub const BIOMETRICS: &str = "/xyz/cyberxero/cyberxero-toolkit/ui/tabs/biometrics.ui";
        pub const CONTAINERS_VMS: &str = "/xyz/cyberxero/cyberxero-toolkit/ui/tabs/containers_vms.ui";
        pub const CUSTOMIZATION: &str = "/xyz/cyberxero/cyberxero-toolkit/ui/tabs/customization.ui";
        pub const DRIVERS: &str = "/xyz/cyberxero/cyberxero-toolkit/ui/tabs/drivers.ui";
        pub const EMULATORS: &str = "/xyz/cyberxero/cyberxero-toolkit/ui/tabs/emulators.ui";
        pub const GAMESCOPE: &str = "/xyz/cyberxero/cyberxero-toolkit/ui/tabs/gamescope.ui";
        pub const GAMING_TOOLS: &str = "/xyz/cyberxero/cyberxero-toolkit/ui/tabs/gaming_tools.ui";
        pub const KERNEL_SCHEDULERS: &str =
            "/xyz/cyberxero/cyberxero-toolkit/ui/tabs/kernel_schedulers.ui";
        pub const MAIN_PAGE: &str = "/xyz/cyberxero/cyberxero-toolkit/ui/tabs/main_page.ui";
        pub const MULTIMEDIA_TOOLS: &str =
            "/xyz/cyberxero/cyberxero-toolkit/ui/tabs/multimedia_tools.ui";
        pub const SERVICING_SYSTEM_TWEAKS: &str =
            "/xyz/cyberxero/cyberxero-toolkit/ui/tabs/servicing_system_tweaks.ui";
    }
}
