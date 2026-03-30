//! Tab navigation and sidebar management.
//!
//! Pages are lazy-loaded **asynchronously** on first visit:
//! 1. Placeholder containers (with a spinner) are added to the stack at startup — zero UI parsing.
//! 2. When a tab is clicked the spinner animates immediately, then `glib::idle_add_local_once`
//!    defers the actual XML parse + handler setup to the next idle cycle.
//! 3. A `loaded_pages` / `loading_pages` guard prevents duplicate loads.
//!
//! Only the first page is loaded eagerly (synchronously at startup) so the user
//! sees real content the moment the window appears.

use crate::ui::pages;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Box as GtkBox, Builder, Button, Image, Label, Orientation, Stack};
use log::{info, warn};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

/// Configuration for a single page in the application.
pub struct PageConfig {
    pub id: &'static str,
    pub title: &'static str,
    pub icon: &'static str,
    pub ui_resource: &'static str,
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
        id: "kernel_schedulers",
        title: "Kernel & Schedulers",
        icon: "hammer-symbolic",
        ui_resource: crate::config::resources::tabs::KERNEL_SCHEDULERS,
        setup_handler: Some(pages::kernel_schedulers::setup_handlers),
    },
    PageConfig {
        id: "servicing_system_tweaks",
        title: "Servicing/System tweaks",
        icon: "toolbox-symbolic",
        ui_resource: crate::config::resources::tabs::SERVICING_SYSTEM_TWEAKS,
        setup_handler: Some(pages::servicing::setup_handlers),
    },
    PageConfig {
        id: "biometrics",
        title: "Biometrics",
        icon: "xfprintd-gui",
        ui_resource: crate::config::resources::tabs::BIOMETRICS,
        setup_handler: Some(pages::biometrics::setup_handlers),
    },
];

// ---------------------------------------------------------------------------
// LazyPageLoader
// ---------------------------------------------------------------------------

/// Tracks which pages have been loaded or are currently loading, and performs
/// the async load when a page is visited for the first time.
pub struct LazyPageLoader {
    loaded_pages: RefCell<HashSet<String>>,
    loading_pages: RefCell<HashSet<String>>,
    main_builder: Builder,
    window: ApplicationWindow,
}

impl LazyPageLoader {
    fn new(main_builder: Builder, window: ApplicationWindow) -> Self {
        Self {
            loaded_pages: RefCell::new(HashSet::new()),
            loading_pages: RefCell::new(HashSet::new()),
            main_builder,
            window,
        }
    }

    fn is_loaded(&self, page_id: &str) -> bool {
        self.loaded_pages.borrow().contains(page_id)
    }

    fn is_loading(&self, page_id: &str) -> bool {
        self.loading_pages.borrow().contains(page_id)
    }

    fn mark_loading(&self, page_id: &str) {
        self.loading_pages.borrow_mut().insert(page_id.to_string());
    }

    /// Trigger an async load for `page_id` if it hasn't been loaded yet.
    ///
    /// The function returns immediately; the actual XML parse happens on the
    /// next GLib idle cycle so the spinner has time to appear first.
    fn ensure_page_loaded(&self, stack: &Stack, page_id: &str) {
        if self.is_loaded(page_id) || self.is_loading(page_id) {
            return;
        }

        let Some(config) = PAGES.iter().find(|p| p.id == page_id) else {
            warn!("Page config not found for: {}", page_id);
            return;
        };

        // Grab the placeholder container that was placed in the stack at startup.
        let Some(child) = stack.child_by_name(page_id) else {
            warn!("Stack child not found for: {}", page_id);
            return;
        };
        let Some(container) = child.downcast_ref::<GtkBox>() else {
            warn!("Stack child is not a GtkBox for: {}", page_id);
            return;
        };

        info!("Scheduling async lazy-load for page '{}'", page_id);
        self.mark_loading(page_id);

        // Start the spinner animation now, before the idle callback fires.
        if let Some(spinner) = find_child_by_name::<Image>(container, "loading_spinner") {
            spinner.add_css_class("spinning");
        }

        // Clone everything needed inside the idle closure.
        let page_id_str = page_id.to_string();
        let ui_resource = config.ui_resource;
        let setup_handler = config.setup_handler;
        let title = config.title;
        let main_builder = self.main_builder.clone();
        let window = self.window.clone();
        let container = container.clone();
        let loaded_pages = self.loaded_pages.clone();
        let loading_pages = self.loading_pages.clone();

        // Defer the heavy work — UI will repaint (showing the spinner) first.
        glib::idle_add_local_once(move || {
            match load_page_content(&page_id_str, ui_resource, setup_handler, &main_builder, &window) {
                Ok(page_widget) => {
                    // Swap out the placeholder for the real page.
                    while let Some(child) = container.first_child() {
                        container.remove(&child);
                    }
                    container.append(&page_widget);

                    loading_pages.borrow_mut().remove(&page_id_str);
                    loaded_pages.borrow_mut().insert(page_id_str.clone());

                    info!("Successfully lazy-loaded page '{}'", page_id_str);
                }
                Err(e) => {
                    warn!("Failed to lazy-load page '{}': {}", page_id_str, e);

                    // Update placeholder to show an error state.
                    if let Some(spinner) = find_child_by_name::<Image>(&container, "loading_spinner") {
                        spinner.remove_css_class("spinning");
                        spinner.set_icon_name(Some("dialog-error-symbolic"));
                    }
                    if let Some(label) = find_child_by_name::<Label>(&container, "loading_label") {
                        label.set_label(&format!("Failed to load {}: {}", title, e));
                    }

                    // Remove from loading but don't add to loaded — lets us retry later.
                    loading_pages.borrow_mut().remove(&page_id_str);
                }
            }
        });
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Recursively search the widget tree for a child with the given name.
fn find_child_by_name<T>(parent: &impl IsA<gtk4::Widget>, name: &str) -> Option<T>
where
    T: IsA<gtk4::Widget> + IsA<glib::Object>,
{
    let parent_widget = parent.upcast_ref::<gtk4::Widget>();
    let mut child = parent_widget.first_child();
    while let Some(widget) = child {
        if widget.widget_name() == name {
            if let Ok(typed) = widget.clone().downcast::<T>() {
                return Some(typed);
            }
        }
        if let Some(found) = find_child_by_name::<T>(&widget, name) {
            return Some(found);
        }
        child = widget.next_sibling();
    }
    None
}

/// Parse a page's UI resource and run its setup handler.
fn load_page_content(
    page_id: &str,
    ui_resource: &str,
    setup_handler: Option<fn(&Builder, &Builder, &ApplicationWindow)>,
    main_builder: &Builder,
    window: &ApplicationWindow,
) -> anyhow::Result<gtk4::Widget> {
    let page_builder = Builder::from_resource(ui_resource);

    let page_widget: gtk4::Widget = page_builder
        .object(format!("page_{}", page_id))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Could not find widget 'page_{}' in resource '{}'",
                page_id,
                ui_resource
            )
        })?;

    if let Some(setup_fn) = setup_handler {
        setup_fn(&page_builder, main_builder, window);
    }

    Ok(page_widget)
}

/// Build a placeholder container shown while a page is loading.
///
/// Visually matches the loading indicator used by the kernel_schedulers page:
/// a spinning refresh icon above a dim "Loading …" label.
fn create_placeholder_container(config: &PageConfig) -> GtkBox {
    let container = GtkBox::new(Orientation::Vertical, 0);
    container.set_hexpand(true);
    container.set_vexpand(true);
    container.set_halign(gtk4::Align::Fill);
    container.set_valign(gtk4::Align::Fill);

    let inner = GtkBox::new(Orientation::Vertical, 12);
    inner.set_hexpand(true);
    inner.set_vexpand(true);
    inner.set_halign(gtk4::Align::Center);
    inner.set_valign(gtk4::Align::Center);

    // Spinner — CSS class "spinning" is added when loading actually starts.
    let spinner = Image::from_icon_name("arrows-rotate-symbolic");
    spinner.set_pixel_size(32);
    spinner.set_widget_name("loading_spinner");

    let label = Label::builder()
        .label(format!("Loading {}...", config.title))
        .halign(gtk4::Align::Center)
        .build();
    label.set_widget_name("loading_label");
    label.add_css_class("dim-label");

    inner.append(&spinner);
    inner.append(&label);
    container.append(&inner);

    container
}

// ---------------------------------------------------------------------------
// Tab
// ---------------------------------------------------------------------------

/// A single entry in the navigation sidebar.
struct Tab {
    page_name: String,
    button: Button,
}

impl Tab {
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

    /// Wire the button click: trigger a lazy-load (if needed) then switch the stack.
    fn connect(&self, stack: &Stack, tabs_container: &GtkBox, loader: &Rc<LazyPageLoader>) {
        let stack_clone = stack.clone();
        let page_name = self.page_name.clone();
        let button_clone = self.button.clone();
        let tabs_clone = tabs_container.clone();
        let loader_clone = Rc::clone(loader);

        self.button.connect_clicked(move |_| {
            info!("Navigating to page '{}'", page_name);

            // Kick off an async load if this is the first visit.
            loader_clone.ensure_page_loaded(&stack_clone, &page_name);

            stack_clone.set_visible_child_name(&page_name);
            update_active_tab(&tabs_clone, &button_clone);
        });
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Build the page stack and sidebar tabs, then return the configured stack.
///
/// All pages except the first get placeholder containers; the first page is
/// loaded eagerly so the user sees real content from the moment the window
/// opens.
pub fn create_stack_and_tabs(tabs_container: &GtkBox, main_builder: &Builder) -> Stack {
    info!("Creating dynamic stack with async lazy loading");

    let window: ApplicationWindow =
        crate::ui::utils::extract_widget(main_builder, "app_window");

    let loader = Rc::new(LazyPageLoader::new(main_builder.clone(), window));

    // Build stack — one placeholder container per page.
    let stack = Stack::new();
    stack.set_hexpand(true);
    stack.set_vexpand(true);
    stack.set_transition_type(gtk4::StackTransitionType::Crossfade);

    for page_config in PAGES {
        let container = create_placeholder_container(page_config);
        stack.add_titled(&container, Some(page_config.id), page_config.title);
        info!("Registered placeholder for page '{}'", page_config.id);
    }

    let right_container =
        crate::ui::utils::extract_widget::<GtkBox>(main_builder, "right_container");
    right_container.append(&stack);

    info!(
        "Dynamic stack created — {} pages registered",
        PAGES.len()
    );

    // Build sidebar tabs.
    let mut first_button: Option<Button> = None;

    for page_config in PAGES {
        let tab = Tab::new(page_config.title, page_config.id, page_config.icon);
        tab.connect(&stack, tabs_container, &loader);

        if first_button.is_none() {
            first_button = Some(tab.button.clone());
        }

        tabs_container.append(&tab.button);
    }

    if let Some(btn) = first_button {
        btn.add_css_class("active");
    }

    // Eagerly load the first page so it's ready when the window opens.
    if let Some(first) = PAGES.first() {
        info!("Eagerly loading first page '{}'", first.id);
        loader.ensure_page_loaded(&stack, first.id);
    }

    stack
}

/// Highlight `clicked_button` and clear the active class from all others.
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
