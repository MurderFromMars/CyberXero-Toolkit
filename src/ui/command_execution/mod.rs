//! Command execution pipeline with progress UI.
//!
//! This module provides a comprehensive command execution system with:
//! - Step-by-step execution status
//! - Live output streaming
//! - Progress tracking
//! - Cancellation support
//! - Privilege escalation handling
//! - AUR helper integration
//!
//! ## Architecture
//!
//! The module is organized into several submodules:
//! - `types`: Command types, steps, and results
//! - `widgets`: UI widget management
//! - `context`: Running command state
//! - `executor`: Command execution logic
//!
//! ## Usage
//!
//! ```no_run
//! use crate::ui::command_execution::{run_commands_with_progress, CommandStep};
//!
//! let commands = vec![
//!     CommandStep::privileged("pacman", &["-Syu"], "System update"),
//!     CommandStep::aur(&["-S", "package-name"], "Install AUR package"),
//! ];
//!
//! run_commands_with_progress(
//!     &parent_window,
//!     commands,
//!     "Installation",
//!     None,
//! );
//! ```

mod context;
mod executor;
mod types;
mod widgets;

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Button, Label, Separator, Window};
use log::{error, warn};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

// Re-export public API
pub use types::CommandStep;

use executor::execute_commands_sequence;
use widgets::{CommandExecutionWidgets, TaskItem};

/// Global flag to track if an action is currently running
static ACTION_RUNNING: AtomicBool = AtomicBool::new(false);

/// Check if an action is currently running.
///
/// This is used to prevent multiple simultaneous operations.
pub fn is_action_running() -> bool {
    ACTION_RUNNING.load(Ordering::SeqCst)
}

/// Show progress dialog and run commands.
///
/// Displays a modal dialog showing command execution progress with:
/// - Progress bar indicating current step
/// - Live output streaming
/// - Cancel and close buttons
/// - Expandable output view
///
/// # Arguments
///
/// * `parent` - Parent window for the dialog
/// * `commands` - Vector of commands to execute
/// * `title` - Dialog title
/// * `on_complete` - Optional callback when all commands complete
///
/// # Example
///
/// ```no_run
/// run_commands_with_progress(
///     &window,
///     vec![CommandStep::normal("ls", &["-la"], "List files")],
///     "File Listing",
///     Some(Box::new(|success| {
///         println!("Completed with success: {}", success);
///     })),
/// );
/// ```
pub fn run_commands_with_progress(
    parent: &Window,
    commands: Vec<CommandStep>,
    title: &str,
    on_complete: Option<Box<dyn Fn(bool) + 'static>>,
) {
    if commands.is_empty() {
        error!("No commands provided");
        return;
    }

    if is_action_running() {
        warn!("Action already running - ignoring request");
        return;
    }

    ACTION_RUNNING.store(true, Ordering::SeqCst);

    // Convert callback to Rc for use across non-Send contexts
    let on_complete = on_complete.map(|cb| Rc::new(cb) as Rc<dyn Fn(bool) + 'static>);

    let builder =
        gtk4::Builder::from_resource("/xyz/xerolinux/xero-toolkit/ui/task_list_dialog.ui");

    let window: Window = builder
        .object("task_window")
        .expect("Failed to get task_window");
    let title_label: Label = builder
        .object("task_title")
        .expect("Failed to get task_title");
    let task_list_container: gtk4::Box = builder
        .object("task_list_container")
        .expect("Failed to get task_list_container");
    let cancel_button: Button = builder
        .object("cancel_button")
        .expect("Failed to get cancel_button");
    let close_button: Button = builder
        .object("close_button")
        .expect("Failed to get close_button");

    window.set_transient_for(Some(parent));
    window.set_title(Some(title));

    // Create task items for each command
    let mut task_items = Vec::new();
    for (i, cmd) in commands.iter().enumerate() {
        let task_item = TaskItem::new(&cmd.friendly_name);
        task_item.set_status(types::TaskStatus::Pending);
        task_list_container.append(&task_item.container);
        // Add separator between tasks (not before the first if preferred)
        if i < commands.len() - 1 {
            let sep = Separator::new(gtk4::Orientation::Horizontal);
            task_list_container.append(&sep);
        }
        task_items.push(task_item);
    }

    let widgets = Rc::new(CommandExecutionWidgets {
        window: window.clone(),
        title_label,
        task_list_container,
        cancel_button: cancel_button.clone(),
        close_button: close_button.clone(),
        task_items,
    });

    let cancelled = Rc::new(RefCell::new(false));
    let current_process = Rc::new(RefCell::new(None::<gtk4::gio::Subprocess>));
    let commands = Rc::new(commands);

    // Cancel button handler
    let widgets_clone = widgets.clone();
    let cancelled_clone = cancelled.clone();
    let running_process = current_process.clone();
    cancel_button.connect_clicked(move |_| {
        *cancelled_clone.borrow_mut() = true;
        widgets_clone.disable_cancel();
        if let Some(process) = running_process.borrow().as_ref() {
            process.force_exit();
        }
    });

    // Close button handler
    let widgets_clone = widgets.clone();
    let on_complete_clone = on_complete.clone();
    close_button.connect_clicked(move |_| {
        widgets_clone.window.close();
        if let Some(ref callback) = on_complete_clone {
            callback(true);
        }
    });

    // Window close handler
    let on_complete_clone = on_complete.clone();
    let current_process_clone = current_process.clone();
    window.connect_close_request(move |_| {
        ACTION_RUNNING.store(false, Ordering::SeqCst);
        if let Some(process) = current_process_clone.borrow().as_ref() {
            process.force_exit();
        }
        if let Some(ref callback) = on_complete_clone {
            callback(false);
        }
        glib::Propagation::Proceed
    });

    window.present();

    // Start executing commands
    execute_commands_sequence(
        widgets,
        commands,
        0,
        cancelled,
        on_complete,
        current_process,
    );
}
