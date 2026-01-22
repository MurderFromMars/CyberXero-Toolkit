//! Tab navigation and sidebar management.
//!
//! This module handles the sidebar navigation tabs that allow users
//! to switch between different pages in the application.

use crate::ui::pages;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Box as GtkBox, Builder, Button, Image, Label, Orientation, Stack};
use log::{info, warn};

/// Configuration for a single page in the application.
pub struct PageConfig {
    /// Internal identifier for the page (used in navigation)
    pub id: &'static str,
    /// Display title for the page
    pub title: &'static str,
    /// Icon name for the tab button
    pub icon: &'static str,
    /// Resource path to the UI file
    pub ui_resource: &'static str,
    /// Function to set up event handlers for the page
    pub setup_handler: Option<fn(&Builder, &Builder, &ApplicationWindow)>,
}

/// Central list of all pages in the application.
/// Comment out any page to disable it entirely.
pub const PAGES: &[PageConfig] = &[
    PageConfig {
        id: "main_page",
        title: "Main Page",
        icon: "house-symbolic",
        ui_resource: crate::config::resources::tabs::MAIN_PAGE,
        setup_handler: Some(pages::main_page::setup_handlers),
    },
    PageConfig {
        id: "drivers",
        title: "Drivers",
        icon: "gear-symbolic",
        ui_resource: crate::config::resources::tabs::DRIVERS,
        setup_handler: Some(pages::drivers::setup_handlers),
    },
    PageConfig {
        id: "customization",
        title: "Customization",
        icon: "brush-symbolic",
        ui_resource: crate::config::resources::tabs::CUSTOMIZATION,
        setup_handler: Some(pages::customization::setup_handlers),
    },
    PageConfig {
        id: "gaming_tools",
        title: "Gaming Tools",
        icon: "gamepad-symbolic",
        ui_resource: crate::config::resources::tabs::GAMING_TOOLS,
        setup_handler: Some(pages::gaming_tools::setup_handlers),
    },
    PageConfig {
        id: "gamescope",
        title: "Gamescope",
        icon: "steam-symbolic",
        ui_resource: crate::config::resources::tabs::GAMESCOPE,
        setup_handler: Some(pages::gamescope::setup_handlers),
    },
    PageConfig {
        id: "containers_vms",
        title: "Containers/VMs",
        icon: "box-symbolic",
        ui_resource: crate::config::resources::tabs::CONTAINERS_VMS,
        setup_handler: Some(pages::containers_vms::setup_handlers),
    },
    PageConfig {
        id: "multimedia_tools",
        title: "Multimedia Tools",
        icon: "play-symbolic",
        ui_resource: crate::config::resources::tabs::MULTIMEDIA_TOOLS,
        setup_handler: Some(pages::multimedia_tools::setup_handlers),
    },
    PageConfig {
        id: "kernel_manager_scx",
        title: "Kernel Manager",
        icon: "hammer-symbolic",
        ui_resource: crate::config::resources::tabs::KERNEL_MANAGER_SCX,
        setup_handler: Some(pages::kernel_manager_scx::setup_handlers),
    },
    PageConfig {
        id: "servicing_system_tweaks",
        title: "Servicing/System tweaks",
        icon: "toolbox-symbolic",
        ui_resource: crate::config::resources::tabs::SERVICING_SYSTEM_TWEAKS,
        setup_handler: Some(pages::servicing::setup_handlers),
    },
];

/// Represents a single tab in the navigation sidebar.
struct Tab {
    page_name: String,
    button: Button,
}

impl Tab {
    /// Create a new tab with the given label, page name and icon.
    fn new(label: &str, page_name: &str, icon_name: &str) -> Self {
        let content_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .hexpand(true)
            .build();

        let image = Image::from_icon_name(icon_name);
        image.set_pixel_size(18);

        let label_widget = Label::new(Some(label));
        label_widget.set_xalign(0.0);

        content_box.append(&image);
        content_box.append(&label_widget);

        let button = Button::builder()
            .hexpand(true)
            .css_classes(vec!["tab-button".to_string()])
            .build();

        button.set_child(Some(&content_box));

        Tab {
            page_name: page_name.to_string(),
            button,
        }
    }

    /// Connect this tab's button to navigate to its page.
    fn connect(&self, stack: &Stack, tabs_container: &GtkBox) {
        let stack_clone = stack.clone();
        let page_name = self.page_name.clone();
        let button_clone = self.button.clone();
        let tabs_clone = tabs_container.clone();

        self.button.connect_clicked(move |_| {
            info!("Navigating to page '{}'", page_name);
            stack_clone.set_visible_child_name(&page_name);
            update_active_tab(&tabs_clone, &button_clone);
        });
    }
}

/// Create dynamic stack with pages and set up navigation tabs.
/// Returns the fully configured stack with all pages and tabs ready.
pub fn create_stack_and_tabs(tabs_container: &GtkBox, main_builder: &Builder) -> Stack {
    info!("Creating dynamic stack and loading pages");

    // Create new stack and populate with pages
    let stack = create_dynamic_stack(main_builder);

    info!("Dynamic stack created with {} pages", PAGES.len());

    // Set up navigation tabs
    info!("Setting up navigation tabs");
    let mut first_button: Option<Button> = None;

    for page_config in PAGES {
        let tab = Tab::new(page_config.title, page_config.id, page_config.icon);
        tab.connect(&stack, tabs_container);

        if first_button.is_none() {
            first_button = Some(tab.button.clone());
        }

        tabs_container.append(&tab.button);
        info!("Added tab: {} -> '{}'", page_config.title, page_config.id);
    }

    // Set first tab as active
    if let Some(button) = first_button {
        button.add_css_class("active");
    }

    stack
}

/// Create a dynamic stack with pages from PAGES configuration.
fn create_dynamic_stack(main_builder: &Builder) -> Stack {
    let stack = Stack::new();
    stack.set_hexpand(true);
    stack.set_vexpand(true);
    stack.set_transition_type(gtk4::StackTransitionType::Crossfade);

    // Dynamically create stack pages from PAGES configuration
    for page_config in PAGES {
        match create_page_from_config(page_config, main_builder) {
            Ok(page_widget) => {
                stack.add_titled(&page_widget, Some(page_config.id), page_config.title);
                info!("Successfully loaded page: {}", page_config.id);
            }
            Err(e) => {
                warn!("Failed to load page '{}': {}", page_config.id, e);
                // Create a fallback page
                let fallback = GtkBox::new(Orientation::Vertical, 0);
                let label = Label::builder()
                    .label(format!("{} page content not available", page_config.title))
                    .build();
                fallback.append(&label);
                stack.add_titled(&fallback, Some(page_config.id), page_config.title);
            }
        }
    }

    // Add the dynamic stack to the right container
    let right_container =
        crate::ui::utils::extract_widget::<GtkBox>(main_builder, "right_container");
    right_container.append(&stack);
    info!("Dynamic stack added to right container");

    stack
}

/// Create a page widget from PageConfig.
fn create_page_from_config(config: &PageConfig, main_builder: &Builder) -> anyhow::Result<GtkBox> {
    use crate::ui::utils::extract_widget;
    use gtk4::ApplicationWindow;

    let page_builder = Builder::from_resource(config.ui_resource);

    let page_widget: gtk4::Widget = page_builder
        .object(format!("page_{}", config.id))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Could not find page_{} widget in {}",
                config.id,
                config.ui_resource
            )
        })?;

    let container = GtkBox::new(Orientation::Vertical, 0);
    container.set_hexpand(true);
    container.set_vexpand(true);
    container.append(&page_widget);

    // Extract main window from builder
    let window: ApplicationWindow = extract_widget(main_builder, "app_window");

    // Call setup handler if provided
    if let Some(setup_fn) = config.setup_handler {
        setup_fn(&page_builder, main_builder, &window);
    }

    Ok(container)
}

/// Update which tab is marked as active.
fn update_active_tab(tabs_container: &GtkBox, clicked_button: &Button) {
    let mut child = tabs_container.first_child();

    while let Some(widget) = child {
        if let Ok(button) = widget.clone().downcast::<Button>() {
            if button == *clicked_button {
                button.add_css_class("active");
            } else {
                button.remove_css_class("active");
            }
        }
        child = widget.next_sibling();
    }
}
