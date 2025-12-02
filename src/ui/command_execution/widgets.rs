//! UI widgets for command execution dialog.

use super::types::TaskStatus;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Window};

/// Container for all command execution dialog widgets
pub struct CommandExecutionWidgets {
    pub window: Window,
    pub title_label: Label,
    #[allow(dead_code)]
    pub task_list_container: GtkBox,
    pub cancel_button: Button,
    pub close_button: Button,
    pub task_items: Vec<TaskItem>,
}

/// A single task item in the task list
pub struct TaskItem {
    pub container: GtkBox,
    pub status_icon: Label,
    pub spinner: gtk4::Spinner,
}

impl TaskItem {
    /// Create a new task item
    pub fn new(description: &str) -> Self {
        let container = GtkBox::new(gtk4::Orientation::Horizontal, 12);
        container.set_margin_top(8);
        container.set_margin_bottom(8);
        container.set_margin_start(12);
        container.set_margin_end(12);

        // Task description label
        let label = Label::new(Some(description));
        label.set_xalign(0.0);
        label.set_hexpand(true);
        label.set_wrap(true);

        // Spinner for running state
        let spinner = gtk4::Spinner::new();
        spinner.set_visible(false);

        // Status icon label
        let status_icon = Label::new(None);
        status_icon.set_visible(false);

        container.append(&label);
        container.append(&spinner);
        container.append(&status_icon);

        Self {
            container,
            status_icon,
            spinner,
        }
    }

    /// Update the status of this task item
    pub fn set_status(&self, status: TaskStatus) {
        match status {
            TaskStatus::Pending => {
                self.spinner.set_visible(false);
                self.status_icon.set_visible(false);
            }
            TaskStatus::Running => {
                self.spinner.set_spinning(true);
                self.spinner.set_visible(true);
                self.status_icon.set_visible(false);
            }
            TaskStatus::Success => {
                self.spinner.set_spinning(false);
                self.spinner.set_visible(false);
                self.status_icon.set_text("✓");
                self.status_icon.set_visible(true);
            }
            TaskStatus::Failed => {
                self.spinner.set_spinning(false);
                self.spinner.set_visible(false);
                self.status_icon.set_text("✗");
                self.status_icon.set_visible(true);
            }
        }
    }
}

impl CommandExecutionWidgets {
    /// Update the status of a specific task
    pub fn update_task_status(&self, index: usize, status: TaskStatus) {
        if let Some(task_item) = self.task_items.get(index) {
            task_item.set_status(status);
        }
    }

    /// Set the dialog title
    pub fn set_title(&self, title: &str) {
        self.title_label.set_text(title);
    }

    /// Disable the cancel button
    pub fn disable_cancel(&self) {
        self.cancel_button.set_sensitive(false);
    }

    /// Enable the close button and hide cancel button
    pub fn enable_close(&self) {
        self.cancel_button.set_visible(false);
        self.close_button.set_visible(true);
        self.close_button.set_sensitive(true);
    }

    /// Show completion state (not needed for task-based UI, kept for compatibility)
    pub fn show_completion(&self, _success: bool, _message: &str) {
        // In task-based UI, completion is shown via task statuses
        // Enable the close button
        self.enable_close();
    }
}
