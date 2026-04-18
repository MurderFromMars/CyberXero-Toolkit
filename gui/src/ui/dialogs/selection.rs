//! Reusable multi-choice picker.
//!
//! Callers build a [`SelectionDialogConfig`] with a list of
//! [`SelectionOption`]s, hand it to [`show_selection_dialog`], and receive
//! the chosen option IDs via the callback once the user hits confirm.

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Builder, Button, CheckButton, Label, Separator, Window};
use log::info;

use crate::ui::utils::extract_widget;

/// One row in the picker.
///
/// `installed` rows are rendered as pre-checked and non-interactive — they
/// communicate "already in place, no action needed" rather than being
/// available for selection.
#[derive(Clone, Debug)]
pub struct SelectionOption {
    pub id: String,
    pub label: String,
    pub description: String,
    pub installed: bool,
}

impl SelectionOption {
    pub fn new(id: &str, label: &str, description: &str, installed: bool) -> Self {
        Self {
            id: id.to_owned(),
            label: label.to_owned(),
            description: description.to_owned(),
            installed,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionType {
    Single,
    Multi,
}

pub struct SelectionDialogConfig {
    pub title: String,
    pub description: String,
    pub options: Vec<SelectionOption>,
    pub confirm_label: String,
    pub selection_type: SelectionType,
    pub selection_required: bool,
}

impl SelectionDialogConfig {
    pub fn new(title: &str, description: &str) -> Self {
        Self {
            title: title.to_owned(),
            description: description.to_owned(),
            options: Vec::new(),
            confirm_label: String::from("Install"),
            selection_type: SelectionType::Multi,
            selection_required: true,
        }
    }

    pub fn add_option(mut self, option: SelectionOption) -> Self {
        self.options.push(option);
        self
    }

    pub fn confirm_label(mut self, label: &str) -> Self {
        self.confirm_label = label.to_owned();
        self
    }

    pub fn selection_type(mut self, selection_type: SelectionType) -> Self {
        self.selection_type = selection_type;
        self
    }

    pub fn selection_required(mut self, required: bool) -> Self {
        self.selection_required = required;
        self
    }
}

/// Internal bookkeeping for a single row. The toggle is whatever widget
/// we created (radio or checkbox, both `CheckButton` in GTK4) paired with
/// the option's caller-facing ID.
struct RowHandle {
    id: String,
    toggle: CheckButton,
}

/// Open the dialog, wire up the toggle logic, and fire `on_confirm` with
/// the selected IDs when the user hits confirm.
pub fn show_selection_dialog<F>(parent: &Window, config: SelectionDialogConfig, on_confirm: F)
where
    F: Fn(Vec<String>) + 'static,
{
    info!("opening selection dialog: {}", config.title);

    let builder = Builder::from_resource(crate::config::resources::dialogs::SELECTION);

    let dialog: Window = extract_widget(&builder, "selection_dialog");
    let title_label: Label = extract_widget(&builder, "dialog_title");
    let description_label: Label = extract_widget(&builder, "dialog_description");
    let options_container: GtkBox = extract_widget(&builder, "options_container");
    let cancel_button: Button = extract_widget(&builder, "cancel_button");
    let confirm_button: Button = extract_widget(&builder, "confirm_button");

    dialog.set_transient_for(Some(parent));
    title_label.set_label(&config.title);
    description_label.set_label(&config.description);
    confirm_button.set_label(&config.confirm_label);

    let selection_type = config.selection_type;
    let selection_required = config.selection_required;

    let rows = Rc::new(RefCell::new(populate_options(
        &options_container,
        &config.options,
        selection_type,
    )));

    apply_confirm_sensitivity(&confirm_button, &rows.borrow(), selection_required);
    wire_sync_on_toggle(&confirm_button, &rows, selection_required);
    wire_cancel(&cancel_button, &dialog);
    wire_confirm(&confirm_button, &dialog, &rows, on_confirm);

    dialog.present();
}

fn populate_options(
    container: &GtkBox,
    options: &[SelectionOption],
    kind: SelectionType,
) -> Vec<RowHandle> {
    let mut rows = Vec::with_capacity(options.len());
    let mut group_anchor: Option<CheckButton> = None;

    for (i, option) in options.iter().enumerate() {
        let toggle = CheckButton::new();
        if matches!(kind, SelectionType::Single) {
            match group_anchor.as_ref() {
                Some(anchor) => toggle.set_group(Some(anchor)),
                None => group_anchor = Some(toggle.clone()),
            }
        }
        toggle.set_active(option.installed);
        toggle.set_sensitive(!option.installed);

        container.append(&build_row(&toggle, option));
        if i + 1 < options.len() {
            container.append(&Separator::new(gtk4::Orientation::Horizontal));
        }

        rows.push(RowHandle {
            id: option.id.clone(),
            toggle,
        });
    }

    rows
}

fn build_row(toggle: &CheckButton, option: &SelectionOption) -> GtkBox {
    let row = GtkBox::new(gtk4::Orientation::Horizontal, 12);
    row.set_margin_start(12);
    row.set_margin_end(12);
    row.set_margin_top(8);
    row.set_margin_bottom(8);

    let text_column = GtkBox::new(gtk4::Orientation::Vertical, 4);
    text_column.set_hexpand(true);

    let title = Label::new(Some(&option.label));
    title.set_halign(Align::Start);
    title.set_wrap(true);
    if option.installed {
        title.set_css_classes(&["dim"]);
    }

    let caption = Label::new(Some(&option.description));
    caption.set_css_classes(&["dim", "caption"]);
    caption.set_halign(Align::Start);
    caption.set_wrap(true);

    text_column.append(&title);
    text_column.append(&caption);

    row.append(toggle);
    row.append(&text_column);
    row
}

fn any_active(rows: &[RowHandle]) -> bool {
    rows.iter().any(|r| r.toggle.is_active())
}

fn apply_confirm_sensitivity(button: &Button, rows: &[RowHandle], required: bool) {
    button.set_sensitive(!required || any_active(rows));
}

fn wire_sync_on_toggle(
    button: &Button,
    rows: &Rc<RefCell<Vec<RowHandle>>>,
    required: bool,
) {
    let button = button.clone();
    let rows_for_sync = rows.clone();
    let sync: Rc<dyn Fn()> = Rc::new(move || {
        button.set_sensitive(!required || any_active(&rows_for_sync.borrow()));
    });

    for row in rows.borrow().iter() {
        let sync = sync.clone();
        row.toggle.connect_toggled(move |_| sync());
    }
}

fn wire_cancel(button: &Button, dialog: &Window) {
    let dialog = dialog.clone();
    button.connect_clicked(move |_| {
        info!("selection cancelled");
        dialog.close();
    });
}

fn wire_confirm<F>(
    button: &Button,
    dialog: &Window,
    rows: &Rc<RefCell<Vec<RowHandle>>>,
    on_confirm: F,
) where
    F: Fn(Vec<String>) + 'static,
{
    let dialog = dialog.clone();
    let rows = rows.clone();
    button.connect_clicked(move |_| {
        let selected: Vec<String> = rows
            .borrow()
            .iter()
            .filter(|r| r.toggle.is_active() && r.toggle.is_sensitive())
            .map(|r| r.id.clone())
            .collect();
        info!("selection confirmed ({} item(s))", selected.len());
        on_confirm(selected);
        dialog.close();
    });
}
