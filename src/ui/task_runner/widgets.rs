//! UI widgets for task runner dialog.

use super::command::TaskStatus;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Image, Label, ScrolledWindow, Window};

/// Container for all task runner dialog widgets.
pub struct TaskRunnerWidgets {
    pub window: Window,
    pub title_label: Label,
    #[allow(dead_code)]
    pub task_list_container: GtkBox,
    pub scrolled_window: ScrolledWindow,
    pub cancel_button: Button,
    pub close_button: Button,
    pub task_items: Vec<TaskItem>,
}

/// A single task item in the task list.
pub struct TaskItem {
    pub container: GtkBox,
    pub status_icon: Image,
    pub spinner_icon: Image,
}

impl TaskItem {
    /// Create a new task item.
    pub fn new(description: &str) -> Self {
        let container = GtkBox::new(gtk4::Orientation::Horizontal, 12);
        container.set_margin_top(12);
        container.set_margin_bottom(12);
        container.set_margin_start(12);
        container.set_margin_end(12);

        let label = Label::new(Some(description));
        label.set_xalign(0.0);
        label.set_hexpand(true);
        label.set_wrap(true);

        // Spinner icon for running state
        let spinner_icon = Image::new();
        spinner_icon.set_icon_name(Some("circle-noth-symbolic"));
        spinner_icon.set_pixel_size(24);
        spinner_icon.set_visible(false);
        spinner_icon.add_css_class("spinning");

        // Status icon for success/failure
        let status_icon = Image::new();
        status_icon.set_pixel_size(24);
        status_icon.set_visible(false);

        container.append(&label);
        container.append(&spinner_icon);
        container.append(&status_icon);

        Self {
            container,
            status_icon,
            spinner_icon,
        }
    }

    /// Update the status of this task item.
    pub fn set_status(&self, status: TaskStatus) {
        match status {
            TaskStatus::Pending => {
                self.spinner_icon.set_visible(false);
                self.status_icon.set_visible(false);
            }
            TaskStatus::Running => {
                self.spinner_icon.set_visible(true);
                self.status_icon.set_visible(false);
            }
            TaskStatus::Success => {
                self.spinner_icon.set_visible(false);
                self.status_icon.set_icon_name(Some("circle-check"));
                self.status_icon.set_visible(true);
            }
            TaskStatus::Failed => {
                self.spinner_icon.set_visible(false);
                self.status_icon.set_icon_name(Some("circle-xmark"));
                self.status_icon.set_visible(true);
            }
            TaskStatus::Cancelled => {
                self.spinner_icon.set_visible(false);
                self.status_icon.set_icon_name(Some("circle-stop"));
                self.status_icon.set_visible(true);
            }
        }
    }
}

impl TaskRunnerWidgets {
    /// Scroll to a specific task in the list (only if outside visible area).
    fn scroll_to_task(&self, index: usize) {
        if self.task_items.get(index).is_none() {
            return;
        }

        let vadjustment = self.scrolled_window.vadjustment();
        let current_scroll = vadjustment.value();
        let page_size = vadjustment.page_size();
        let upper = vadjustment.upper();

        let total_tasks = self.task_items.len() as f64;
        let content_height = upper;
        let task_height = content_height / total_tasks;

        let task_top = (index as f64) * task_height;
        let task_bottom = task_top + task_height;

        let viewport_top = current_scroll;
        let viewport_bottom = current_scroll + page_size;

        if task_bottom > viewport_bottom {
            let target_value = (task_bottom - page_size).max(0.0).min(upper - page_size);
            vadjustment.set_value(target_value);
        } else if task_top < viewport_top {
            let target_value = task_top.max(0.0);
            vadjustment.set_value(target_value);
        }
    }

    /// Update the status of a specific task.
    pub fn update_task_status(&self, index: usize, status: TaskStatus) {
        if let Some(task_item) = self.task_items.get(index) {
            task_item.set_status(status);
            self.scroll_to_task(index);
        }
    }

    /// Set the dialog title.
    pub fn set_title(&self, title: &str) {
        self.title_label.set_text(title);
    }

    /// Disable the cancel button.
    pub fn disable_cancel(&self) {
        self.cancel_button.set_sensitive(false);
    }

    /// Enable the close button and hide cancel button.
    pub fn enable_close(&self) {
        self.cancel_button.set_visible(false);
        self.close_button.set_visible(true);
        self.close_button.set_sensitive(true);
    }

    /// Show completion state.
    #[allow(unused_variables)]
    pub fn show_completion(&self, success: bool, message: &str) {
        self.set_title(message);
        self.enable_close();
    }
}
