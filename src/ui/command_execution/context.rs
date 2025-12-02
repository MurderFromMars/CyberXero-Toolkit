//! Running command context and state management.

use super::executor::{execute_commands_sequence, finalize_execution};
use super::types::{CommandResult, CommandStep};
use super::widgets::CommandExecutionWidgets;
use gtk4::gio;
use std::cell::RefCell;
use std::rc::Rc;

/// Context for a running command execution
pub struct RunningCommandContext {
    pub widgets: Rc<CommandExecutionWidgets>,
    pub commands: Rc<Vec<CommandStep>>,
    pub index: usize,
    pub cancelled: Rc<RefCell<bool>>,
    pub on_complete: Option<Rc<dyn Fn(bool) + 'static>>,
    pub current_process: Rc<RefCell<Option<gio::Subprocess>>>,
    exit_result: RefCell<Option<CommandResult>>,
}

impl RunningCommandContext {
    /// Create a new running command context
    pub fn new(
        widgets: Rc<CommandExecutionWidgets>,
        commands: Rc<Vec<CommandStep>>,
        index: usize,
        cancelled: Rc<RefCell<bool>>,
        on_complete: Option<Rc<dyn Fn(bool) + 'static>>,
        current_process: Rc<RefCell<Option<gio::Subprocess>>>,
    ) -> Rc<Self> {
        Rc::new(Self {
            widgets,
            commands,
            index,
            cancelled,
            on_complete,
            current_process,
            exit_result: RefCell::new(None),
        })
    }

    /// Set the exit result for the current command
    pub fn set_exit_result(self: &Rc<Self>, result: CommandResult) {
        *self.exit_result.borrow_mut() = Some(result);
        self.try_finalize();
    }

    /// Try to finalize the current command
    fn try_finalize(self: &Rc<Self>) {
        let result = {
            let mut exit_result = self.exit_result.borrow_mut();
            exit_result.take()
        };

        let Some(result) = result else {
            return;
        };

        // Clear current process
        self.current_process.borrow_mut().take();

        // Check if cancelled
        if *self.cancelled.borrow() {
            finalize_execution(&self.widgets, false, "Operation cancelled");
            if let Some(callback) = &self.on_complete {
                callback(false);
            }
            return;
        }

        // Handle result
        match result {
            CommandResult::Success => {
                // Mark task as successful
                self.widgets
                    .update_task_status(self.index, super::types::TaskStatus::Success);
                execute_commands_sequence(
                    self.widgets.clone(),
                    self.commands.clone(),
                    self.index + 1,
                    self.cancelled.clone(),
                    self.on_complete.clone(),
                    self.current_process.clone(),
                );
            }
            CommandResult::Failure { exit_code: _ } => {
                // Mark task as failed
                self.widgets
                    .update_task_status(self.index, super::types::TaskStatus::Failed);
                finalize_execution(
                    &self.widgets,
                    false,
                    &format!(
                        "Operation failed at step {} of {}",
                        self.index + 1,
                        self.commands.len()
                    ),
                );
                if let Some(callback) = &self.on_complete {
                    callback(false);
                }
            }
        }
    }

    /// Check if the context is cancelled
    #[allow(dead_code)]
    pub fn is_cancelled(&self) -> bool {
        *self.cancelled.borrow()
    }

    /// Get the current command index
    #[allow(dead_code)]
    pub fn current_index(&self) -> usize {
        self.index
    }

    /// Get the total number of commands
    #[allow(dead_code)]
    pub fn total_commands(&self) -> usize {
        self.commands.len()
    }
}
