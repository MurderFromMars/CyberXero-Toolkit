//! Application setup and initialization.

use crate::config;
use crate::core;
use crate::ui::context::AppContext;
use crate::ui::context::UiComponents;
use crate::ui::navigation;
use crate::ui::utils::extract_widget;
use adw::prelude::*;
use adw::Application;
use gtk4::glib;
use gtk4::{gio, ApplicationWindow, Builder, CssProvider, Stack};
use log::{error, info, warn};

/// Initialize and set up main application UI.
pub fn setup_application_ui(app: &Application) {
    info!("Initializing application components");

    setup_resources_and_theme();

    let builder = Builder::from_resource(config::resources::MAIN_UI);
    let window = create_main_window(app, &builder);

    // Initialize environment variables before building UI
    // (some page handlers need USER/HOME)
    info!("Initializing environment variables");
    if let Err(e) = config::env::init() {
        error!("Failed to initialize environment variables: {}", e);
        window.present();
        crate::ui::dialogs::error::show_error(&window, &format!("Failed to initialize environment variables: {}\n\nRequired environment variables (USER, HOME) are not set.", e));
        return;
    }

    // Extract tabs_container first for stack creation
    let tabs_container = extract_widget(&builder, "tabs_container");

    // Create dynamic stack with all pages and set up navigation tabs
    let stack = navigation::create_stack_and_tabs(&tabs_container, &builder);

    // Set up UI components with the dynamic stack
    let ctx = setup_ui_components(&builder, stack, &window);

    info!("Setting initial view to first page");
    if let Some(first_page) = navigation::PAGES.first() {
        ctx.navigate_to_page(first_page.id);
    }

    // Apply seasonal effects (snow for December, Halloween for October, etc.)
    crate::ui::seasonal::apply_seasonal_effects(&window);

    // Present the window only after the full UI is assembled —
    // this prevents the visible resize/hitch where the window
    // appears empty at a small size before the WM tiles it.
    window.present();

    // Perform system checks off the main thread so they don't block
    // window rendering. Results are sent back via a GLib channel.
    let (sender, receiver) = glib::MainContext::channel::<(bool, Option<core::system_check::DependencyCheckResult>, bool)>(glib::Priority::DEFAULT);

    std::thread::spawn(move || {
        info!("Checking system dependencies (background thread)");

        let is_xero = core::system_check::check_xerolinux_distribution();
        let (dep_result, aur_ok) = if is_xero {
            let deps = core::system_check::check_dependencies();
            let aur = if !deps.has_missing_dependencies() {
                core::aur::init()
            } else {
                false
            };
            (Some(deps), aur)
        } else {
            (None, false)
        };

        let _ = sender.send((is_xero, dep_result, aur_ok));
    });

    let window_clone = window.clone();
    receiver.attach(None, move |(is_xero, dep_result, aur_ok)| {
        if !is_xero {
            warn!("Dependency check failed - not running on XeroLinux");
            core::system_check::show_xerolinux_error_dialog(&window_clone);
        } else if let Some(ref result) = dep_result {
            if result.has_missing_dependencies() {
                warn!("Dependency check failed - missing dependencies");
                core::system_check::show_dependency_error_dialog(&window_clone, result);
            } else {
                if aur_ok {
                    info!("AUR helper initialized successfully");
                }
                info!("All dependency checks passed");
            }
        }
        glib::ControlFlow::Break
    });

    info!("Xero Toolkit application startup complete");
}

/// Set up resources and theme.
fn setup_resources_and_theme() {
    info!("Setting up resources and theme");

    gio::resources_register_include!("xyz.xerolinux.xero-toolkit.gresource")
        .expect("Failed to register gresources");

    if let Some(display) = gtk4::gdk::Display::default() {
        info!("Setting up UI theme and styling");

        let theme = gtk4::IconTheme::for_display(&display);
        // Don't inherit system icon themes
        theme.set_search_path(&[]);
        theme.add_resource_path(config::resources::ICONS);
        info!("Icon theme paths configured");

        let css_provider = CssProvider::new();
        css_provider.load_from_resource(config::resources::CSS);
        gtk4::style_context_add_provider_for_display(
            &display,
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        info!("UI theme and styling loaded successfully");
    } else {
        warn!("No default display found - UI theming may not work properly");
    }
}

/// Create main application window.
fn create_main_window(app: &Application, builder: &Builder) -> ApplicationWindow {
    let window: ApplicationWindow = extract_widget(builder, "app_window");

    window.set_application(Some(app));
    info!("Setting window icon to xero-toolkit");
    window.set_icon_name(Some("xero-toolkit"));
    info!("Main application window created from UI resource");

    window
}

/// Set up UI components and return application context.
fn setup_ui_components(builder: &Builder, stack: Stack, window: &ApplicationWindow) -> AppContext {
    let tabs_container = extract_widget(builder, "tabs_container");
    let main_split_view = extract_widget(builder, "main_split_view");
    let sidebar_toggle = extract_widget(builder, "sidebar_toggle_button");

    // Set up autostart toggle in sidebar
    setup_autostart_toggle(builder);

    // Set up about button
    setup_about_button(builder, window);

    // Set up seasonal effects toggle
    setup_seasonal_effects_toggle(builder, window);

    info!("All UI components successfully initialized from UI builder");

    let ui = UiComponents::new(stack, tabs_container, main_split_view, sidebar_toggle);

    // Configure sidebar with size constraints from config
    ui.configure_sidebar(config::sidebar::MIN_WIDTH, config::sidebar::MAX_WIDTH);

    AppContext::new(ui)
}

/// Set up the autostart toggle switch in the sidebar.
fn setup_autostart_toggle(builder: &Builder) {
    let switch = extract_widget::<gtk4::Switch>(builder, "switch_autostart");
    // Set initial state based on whether autostart is enabled
    switch.set_active(core::autostart::is_enabled());

    switch.connect_state_set(move |_switch, state| {
        info!("Autostart toggle changed to: {}", state);

        let result = if state {
            core::autostart::enable()
        } else {
            core::autostart::disable()
        };

        if let Err(e) = result {
            warn!(
                "Failed to {} autostart: {}",
                if state { "enable" } else { "disable" },
                e
            );
            // Return Propagation::Stop to prevent the switch from updating its state
            return glib::Propagation::Stop;
        }

        // Return Propagation::Proceed to allow the switch to update its state
        glib::Propagation::Proceed
    });
}

/// Set up the about button in the header bar.
fn setup_about_button(builder: &Builder, window: &ApplicationWindow) {
    use crate::ui::dialogs::about;

    let button = extract_widget::<gtk4::Button>(builder, "about_button");
    let window_clone = window.clone();
    button.connect_clicked(move |_| {
        info!("About button clicked");
        about::show_about_dialog(window_clone.upcast_ref());
    });
}

/// Set up the seasonal effects toggle button in the header bar.
fn setup_seasonal_effects_toggle(builder: &Builder, _window: &ApplicationWindow) {
    use crate::ui::seasonal;

    let toggle = extract_widget::<gtk4::ToggleButton>(builder, "seasonal_effects_toggle");

    // Show/hide button based on whether any effect is active
    let has_active = seasonal::has_active_effect();
    toggle.set_visible(has_active);
    toggle.set_active(seasonal::are_effects_enabled());

    // Connect toggle action
    toggle.connect_toggled(move |btn| {
        let enabled = btn.is_active();
        seasonal::set_effects_enabled(enabled);
        info!(
            "Seasonal effects {}",
            if enabled { "enabled" } else { "disabled" }
        );
    });
}
