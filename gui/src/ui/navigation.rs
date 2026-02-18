//! Tab navigation and sidebar management.
//!
//! Pages are initialized **fully lazily** — neither the UI XML nor the
//! setup handlers are loaded until the user first navigates to a page.
//! Only the initial (first) page is loaded eagerly. This avoids parsing
//! 10 UI files and spawning dozens of subprocess checks at startup.

use crate::ui::pages;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, Box as GtkBox, Builder, Button, Image, Label, Orientation, Stack};
use log::{info, warn};
use std::cell::RefCell;
use std::collections::HashMap;
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

/// Everything needed to lazily load a page on first visit.
struct PendingPage {
    ui_resource: &'static str,
    setup_fn: Option<fn(&Builder, &Builder, &ApplicationWindow)>,
    /// The empty container sitting in the stack — we'll populate it on first visit.
    container: GtkBox,
}

type PendingMap = Rc<RefCell<HashMap<String, PendingPage>>>;

/// Represents a single tab in the navigation sidebar.
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

    /// Connect this tab's button to navigate to its page.
    /// On first visit, loads the page UI from resources and runs setup_handler.
    fn connect(
        &self,
        stack: &Stack,
        tabs_container: &GtkBox,
        pending: &PendingMap,
        main_builder: &Builder,
    ) {
        let stack_clone = stack.clone();
        let page_name = self.page_name.clone();
        let button_clone = self.button.clone();
        let tabs_clone = tabs_container.clone();
        let pending_clone = Rc::clone(pending);
        let main_builder_clone = main_builder.clone();

        self.button.connect_clicked(move |_| {
            // Lazy-load on first visit: parse UI XML + run setup handler
            if let Some(pending_page) = pending_clone.borrow_mut().remove(&page_name) {
                info!("Lazy-loading page '{}' on first visit", page_name);
                load_pending_page(&page_name, pending_page, &main_builder_clone);
            }

            stack_clone.set_visible_child_name(&page_name);
            update_active_tab(&tabs_clone, &button_clone);
        });
    }
}

/// Populate a pending page's container with the actual UI and run its handler.
fn load_pending_page(page_id: &str, pending: PendingPage, main_builder: &Builder) {
    let page_builder = Builder::from_resource(pending.ui_resource);

    let widget_id = format!("page_{}", page_id);
    match page_builder.object::<gtk4::Widget>(&widget_id) {
        Some(page_widget) => {
            pending.container.append(&page_widget);

            if let Some(setup_fn) = pending.setup_fn {
                let window: ApplicationWindow =
                    crate::ui::utils::extract_widget(main_builder, "app_window");
                setup_fn(&page_builder, main_builder, &window);
            }
        }
        None => {
            warn!(
                "Could not find widget '{}' in {}",
                widget_id, pending.ui_resource
            );
            let label = Label::builder()
                .label(format!("Page content not available"))
                .build();
            pending.container.append(&label);
        }
    }
}

/// Create dynamic stack with pages and set up navigation tabs.
pub fn create_stack_and_tabs(tabs_container: &GtkBox, main_builder: &Builder) -> Stack {
    info!("Creating dynamic stack and loading pages");

    let pending: PendingMap = Rc::new(RefCell::new(HashMap::new()));
    let stack = Stack::new();
    stack.set_hexpand(true);
    stack.set_vexpand(true);
    stack.set_transition_type(gtk4::StackTransitionType::Crossfade);

    let mut is_first = true;

    for page_config in PAGES {
        let container = GtkBox::new(Orientation::Vertical, 0);
        container.set_hexpand(true);
        container.set_vexpand(true);

        if is_first {
            // First page — load eagerly so the user sees content immediately
            is_first = false;
            let page_builder = Builder::from_resource(page_config.ui_resource);

            if let Some(page_widget) =
                page_builder.object::<gtk4::Widget>(&format!("page_{}", page_config.id))
            {
                container.append(&page_widget);
                if let Some(setup_fn) = page_config.setup_handler {
                    let window: ApplicationWindow =
                        crate::ui::utils::extract_widget(main_builder, "app_window");
                    setup_fn(&page_builder, main_builder, &window);
                }
            }
            info!("Loaded page {} (eagerly)", page_config.id);
        } else {
            // All other pages — fully deferred (no UI parsing until first visit)
            pending.borrow_mut().insert(
                page_config.id.to_string(),
                PendingPage {
                    ui_resource: page_config.ui_resource,
                    setup_fn: page_config.setup_handler,
                    container: container.clone(),
                },
            );
            info!("Registered page {} (lazy)", page_config.id);
        }

        stack.add_titled(&container, Some(page_config.id), page_config.title);
    }

    // Add the dynamic stack to the right container
    let right_container =
        crate::ui::utils::extract_widget::<GtkBox>(main_builder, "right_container");
    right_container.append(&stack);

    info!("Dynamic stack created — 1 eager, {} lazy", PAGES.len() - 1);

    // Set up navigation tabs
    let mut first_button: Option<Button> = None;

    for page_config in PAGES {
        let tab = Tab::new(page_config.title, page_config.id, page_config.icon);
        tab.connect(&stack, tabs_container, &pending, main_builder);

        if first_button.is_none() {
            first_button = Some(tab.button.clone());
        }

        tabs_container.append(&tab.button);
    }

    if let Some(button) = first_button {
        button.add_css_class("active");
    }

    stack
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
