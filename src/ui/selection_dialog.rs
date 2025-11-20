//! Selection dialog for multi-choice options.
//!
//! This module provides a reusable dialog window for presenting users with
//! multiple options to select from, with customizable title, description, and actions.

use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Builder, Button, CheckButton, Label, Window};
use log::info;
use std::rc::Rc;
use std::cell::RefCell;

/// Represents a selectable option in the dialog
#[derive(Clone, Debug)]
pub struct SelectionOption {
    pub id: String,
    pub label: String,
    pub description: String,
    pub installed: bool,
}

impl SelectionOption {
    /// Create a new selection option
    pub fn new(id: &str, label: &str, description: &str, installed: bool) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            description: description.to_string(),
            installed,
        }
    }
}

/// Configuration for the selection dialog
pub struct SelectionDialogConfig {
    pub title: String,
    pub description: String,
    pub options: Vec<SelectionOption>,
    pub confirm_label: String,
}

impl SelectionDialogConfig {
    /// Create a new dialog configuration
    pub fn new(title: &str, description: &str) -> Self {
        Self {
            title: title.to_string(),
            description: description.to_string(),
            options: Vec::new(),
            confirm_label: "Install".to_string(),
        }
    }

    /// Add an option to the dialog
    pub fn add_option(mut self, option: SelectionOption) -> Self {
        self.options.push(option);
        self
    }

    /// Set the confirm button label
    pub fn confirm_label(mut self, label: &str) -> Self {
        self.confirm_label = label.to_string();
        self
    }
}

/// Show a selection dialog and call the callback with selected option IDs
pub fn show_selection_dialog<F>(
    parent: &Window,
    config: SelectionDialogConfig,
    on_confirm: F,
) where
    F: Fn(Vec<String>) + 'static,
{
    info!("Opening selection dialog: {}", config.title);

    // Load the UI from resource
    let builder = Builder::from_resource("/xyz/xerolinux/xero-toolkit/ui/selection_dialog.ui");

    // Get the dialog window
    let dialog: Window = builder
        .object("selection_dialog")
        .expect("Failed to get selection_dialog");

    // Set transient parent
    dialog.set_transient_for(Some(parent));

    // Get UI elements
    let title_label: Label = builder
        .object("dialog_title")
        .expect("Failed to get dialog_title");
    let description_label: Label = builder
        .object("dialog_description")
        .expect("Failed to get dialog_description");
    let options_container: GtkBox = builder
        .object("options_container")
        .expect("Failed to get options_container");
    let cancel_button: Button = builder
        .object("cancel_button")
        .expect("Failed to get cancel_button");
    let confirm_button: Button = builder
        .object("confirm_button")
        .expect("Failed to get confirm_button");

    // Set title and description
    title_label.set_label(&config.title);
    description_label.set_label(&config.description);
    confirm_button.set_label(&config.confirm_label);

    let checkboxes: Rc<RefCell<Vec<(String, CheckButton)>>> = Rc::new(RefCell::new(Vec::new()));

    let available_options: Vec<_> = config.options.into_iter()
        .filter(|opt| !opt.installed)
        .collect();

    if available_options.is_empty() {
        // All options are already installed
        let no_options_label = Label::new(Some("All options are already installed."));
        no_options_label.set_css_classes(&["dim"]);
        options_container.append(&no_options_label);

        // Disable confirm button
        confirm_button.set_sensitive(false);
    } else {
        for option in available_options {
            let option_box = GtkBox::new(gtk4::Orientation::Vertical, 4);
            option_box.set_margin_start(12);
            option_box.set_margin_end(12);

            let checkbox = CheckButton::with_label(&option.label);
            checkbox.set_active(false);

            let desc_label = Label::new(Some(&option.description));
            desc_label.set_css_classes(&["dim", "caption"]);
            desc_label.set_halign(gtk4::Align::Start);
            desc_label.set_margin_start(24);
            desc_label.set_wrap(true);

            option_box.append(&checkbox);
            option_box.append(&desc_label);

            options_container.append(&option_box);

            checkboxes.borrow_mut().push((option.id, checkbox));
        }
    }

    // Cancel button - just close the dialog
    let dialog_clone = dialog.clone();
    cancel_button.connect_clicked(move |_| {
        info!("Selection dialog cancelled");
        dialog_clone.close();
    });

    // Confirm button - collect selected options and call callback
    let dialog_clone = dialog.clone();
    let checkboxes_clone = checkboxes.clone();
    confirm_button.connect_clicked(move |_| {
        let selected: Vec<String> = checkboxes_clone
            .borrow()
            .iter()
            .filter(|(_, checkbox)| checkbox.is_active())
            .map(|(id, _)| id.clone())
            .collect();

        info!("Selection dialog confirmed with {} selections", selected.len());

        if !selected.is_empty() {
            on_confirm(selected);
        }

        dialog_clone.close();
    });

    // Show the dialog
    dialog.present();
}

