//! Application context and state management.

use gtk4::{Box as GtkBox, Paned, Stack};

/// Main application context with UI elements.
#[derive(Clone)]
pub struct AppContext {
    pub ui: UiComponents,
}

/// UI components grouped by functionality.
#[derive(Clone)]
pub struct UiComponents {
    pub stack: Stack,
    pub tabs_container: GtkBox,
    #[allow(dead_code)]
    pub main_paned: Paned,
}

