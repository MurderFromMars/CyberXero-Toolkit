//! Command execution logic and running context management.

use super::command::{Command, CommandResult, CommandType, TaskStatus};
use super::widgets::TaskRunnerWidgets;
use crate::core;
use gtk4::gio;
use log::{error, info};
use std::cell::RefCell;
use std::ffi::OsString;
use std::rc::Rc;

/// Context for a running command execution.
pub struct RunningContext {
    pub widgets: Rc<TaskRunnerWidgets>,
    pub commands: Rc<Vec<Command>>,
    pub index: usize,
    pub cancelled: Rc<RefCell<bool>>,
    pub on_complete: Option<Rc<dyn Fn(bool) + 'static>>,
    pub current_process: Rc<RefCell<Option<gio::Subprocess>>>,
    exit_result: RefCell<Option<CommandResult>>,
}

impl RunningContext {
    /// Create a new running command context.
    pub fn new(
        widgets: Rc<TaskRunnerWidgets>,
        commands: Rc<Vec<Command>>,
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

    /// Set the exit result for the current command.
    pub fn set_exit_result(self: &Rc<Self>, result: CommandResult) {
        *self.exit_result.borrow_mut() = Some(result);
        self.try_finalize();
    }

    /// Try to finalize the current command.
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
            // Mark the current task as cancelled
            self.widgets
                .update_task_status(self.index, TaskStatus::Cancelled);
            finalize_execution(&self.widgets, false, "Operation cancelled by user");
            if let Some(callback) = &self.on_complete {
                callback(false);
            }
            return;
        }

        // Handle result
        match result {
            CommandResult::Success => {
                self.widgets
                    .update_task_status(self.index, TaskStatus::Success);
                execute_commands(
                    self.widgets.clone(),
                    self.commands.clone(),
                    self.index + 1,
                    self.cancelled.clone(),
                    self.on_complete.clone(),
                    self.current_process.clone(),
                );
            }
            CommandResult::Failure { .. } => {
                self.widgets
                    .update_task_status(self.index, TaskStatus::Failed);
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
}

/// Execute a sequence of commands.
pub fn execute_commands(
    widgets: Rc<TaskRunnerWidgets>,
    commands: Rc<Vec<Command>>,
    index: usize,
    cancelled: Rc<RefCell<bool>>,
    on_complete: Option<Rc<dyn Fn(bool) + 'static>>,
    current_process: Rc<RefCell<Option<gio::Subprocess>>>,
) {
    if *cancelled.borrow() {
        // If there's a current task being processed, mark it as cancelled
        if index < commands.len() {
            widgets.update_task_status(index, TaskStatus::Cancelled);
        }
        finalize_execution(&widgets, false, "Operation cancelled by user");
        if let Some(callback) = on_complete {
            callback(false);
        }
        return;
    }

    if index >= commands.len() {
        finalize_execution(&widgets, true, "All operations completed successfully!");
        if let Some(callback) = on_complete {
            callback(true);
        }
        return;
    }

    let cmd = &commands[index];

    // Mark current task as running
    widgets.update_task_status(index, TaskStatus::Running);
    widgets.set_title(&cmd.description);

    let (program, args) = match resolve_command(cmd) {
        Ok(result) => result,
        Err(err) => {
            error!("Failed to prepare command: {}", err);
            widgets.update_task_status(index, TaskStatus::Failed);
            finalize_execution(
                &widgets,
                false,
                &format!("Failed to prepare command: {}", err),
            );
            if let Some(callback) = on_complete {
                callback(false);
            }
            return;
        }
    };

    info!("Executing: {} {:?}", program, args);

    let mut argv: Vec<OsString> = Vec::with_capacity(1 + args.len());
    argv.push(OsString::from(program.clone()));
    for arg in &args {
        argv.push(OsString::from(arg));
    }
    let argv_refs: Vec<&std::ffi::OsStr> = argv.iter().map(|s| s.as_os_str()).collect();

    let flags = gio::SubprocessFlags::empty();
    let subprocess = match gio::Subprocess::newv(&argv_refs, flags) {
        Ok(proc) => proc,
        Err(err) => {
            error!("Failed to start command: {}", err);
            widgets.update_task_status(index, TaskStatus::Failed);
            finalize_execution(
                &widgets,
                false,
                &format!("Failed to start operation: {}", err),
            );
            if let Some(callback) = on_complete {
                callback(false);
            }
            return;
        }
    };

    *current_process.borrow_mut() = Some(subprocess.clone());

    let context = RunningContext::new(
        widgets.clone(),
        commands.clone(),
        index,
        cancelled.clone(),
        on_complete.clone(),
        current_process.clone(),
    );

    let wait_context = context.clone();
    let wait_subprocess = subprocess.clone();
    wait_subprocess
        .clone()
        .wait_async(None::<&gio::Cancellable>, move |result| match result {
            Ok(_) => {
                if wait_subprocess.is_successful() {
                    wait_context.set_exit_result(CommandResult::Success);
                } else {
                    wait_context.set_exit_result(CommandResult::Failure {
                        exit_code: Some(wait_subprocess.exit_status()),
                    });
                }
            }
            Err(err) => {
                error!("Failed to wait for command: {}", err);
                wait_context.set_exit_result(CommandResult::Failure { exit_code: None });
            }
        });
}

/// Resolve command with proper privilege escalation and AUR helpers.
fn resolve_command(command: &Command) -> Result<(String, Vec<String>), String> {
    match command.command_type {
        CommandType::Normal => Ok((command.program.clone(), command.args.clone())),
        CommandType::Privileged => {
            let mut args = Vec::with_capacity(command.args.len() + 1);
            args.push(command.program.clone());
            args.extend(command.args.clone());
            Ok(("pkexec".to_string(), args))
        }
        CommandType::Aur => {
            let helper = core::aur_helper()
                .ok_or_else(|| "AUR helper not available (paru or yay required)".to_string())?;
            let mut args = Vec::with_capacity(command.args.len() + 2);
            args.push("--sudo".to_string());
            args.push("pkexec".to_string());
            args.extend(command.args.clone());
            Ok((helper.to_string(), args))
        }
    }
}

/// Finalize dialog with success or failure message.
pub fn finalize_execution(widgets: &TaskRunnerWidgets, success: bool, message: &str) {
    use std::sync::atomic::Ordering;
    super::ACTION_RUNNING.store(false, Ordering::SeqCst);

    widgets.show_completion(success, message);
}
