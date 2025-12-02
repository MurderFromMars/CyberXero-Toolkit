//! Command execution logic and stream handling.

use super::context::RunningCommandContext;
use super::types::{CommandResult, CommandStep, CommandType};
use super::widgets::CommandExecutionWidgets;
use crate::{aur_helper, utils};
use gtk4::gio;
use log::{error, info};
use std::cell::RefCell;
use std::ffi::OsString;
use std::rc::Rc;

/// Execute a sequence of commands
pub fn execute_commands_sequence(
    widgets: Rc<CommandExecutionWidgets>,
    commands: Rc<Vec<CommandStep>>,
    index: usize,
    cancelled: Rc<RefCell<bool>>,
    on_complete: Option<Rc<dyn Fn(bool) + 'static>>,
    current_process: Rc<RefCell<Option<gio::Subprocess>>>,
) {
    if *cancelled.borrow() {
        finalize_execution(&widgets, false, "Operation cancelled");
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
    widgets.update_task_status(index, super::types::TaskStatus::Running);
    widgets.set_title(&cmd.friendly_name);

    let (full_command, full_args) = match resolve_command(cmd) {
        Ok(result) => result,
        Err(err) => {
            error!("Failed to prepare command: {}", err);
            finalize_execution(&widgets, false, "Failed to prepare command");
            if let Some(callback) = on_complete {
                callback(false);
            }
            return;
        }
    };

    info!("Executing: {} {:?}", full_command, full_args);

    let mut argv: Vec<OsString> = Vec::with_capacity(1 + full_args.len());
    argv.push(OsString::from(full_command.clone()));
    for arg in &full_args {
        argv.push(OsString::from(arg));
    }
    let argv_refs: Vec<&std::ffi::OsStr> = argv.iter().map(|s| s.as_os_str()).collect();

    // We don't capture output anymore, so no pipes needed
    let flags = gio::SubprocessFlags::empty();
    let subprocess = match gio::Subprocess::newv(&argv_refs, flags) {
        Ok(proc) => proc,
        Err(err) => {
            error!("Failed to start command: {}", err);
            finalize_execution(&widgets, false, "Failed to start operation");
            if let Some(callback) = on_complete {
                callback(false);
            }
            return;
        }
    };

    *current_process.borrow_mut() = Some(subprocess.clone());

    let context = RunningCommandContext::new(
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

/// Resolve command with proper privilege escalation and helpers
pub fn resolve_command(command: &CommandStep) -> Result<(String, Vec<String>), String> {
    match command.command_type {
        CommandType::Normal => Ok((command.command.clone(), command.args.clone())),
        CommandType::Privileged => {
            let mut args = Vec::with_capacity(command.args.len() + 1);
            args.push(command.command.clone());
            args.extend(command.args.clone());
            Ok(("pkexec".to_string(), args))
        }
        CommandType::Aur => {
            let helper = aur_helper()
                .map(|h| h.to_string())
                .or_else(|| utils::detect_aur_helper().map(|h| h.to_string()))
                .ok_or_else(|| "AUR helper not initialized (paru or yay required).".to_string())?;
            let mut args = Vec::with_capacity(command.args.len() + 2);
            args.push("--sudo".to_string());
            args.push("pkexec".to_string());
            args.extend(command.args.clone());
            Ok((helper, args))
        }
    }
}

/// Finalize dialog with success or failure message
pub fn finalize_execution(widgets: &CommandExecutionWidgets, success: bool, message: &str) {
    use std::sync::atomic::Ordering;
    super::ACTION_RUNNING.store(false, Ordering::SeqCst);

    widgets.show_completion(success, message);
}
