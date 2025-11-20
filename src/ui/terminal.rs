//! Terminal command execution functionality.
//!
//! This module handles running interactive terminal commands using VTE (Virtual Terminal Emulator)
//! with PTY support, allowing for password prompts, user input, and real-time output streaming.

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{gio, Button, Label};
use log::{error, info};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use vte4::prelude::*;
use vte4::Terminal;

/// Global flag to track if an action is currently running
static ACTION_RUNNING: AtomicBool = AtomicBool::new(false);

thread_local! {
    /// Store the signal handler ID for child-exited to disconnect old handlers
    static CHILD_EXITED_HANDLER: RefCell<Option<glib::SignalHandlerId>> = RefCell::new(None);
}

/// Check if an action is currently running
pub fn is_action_running() -> bool {
    ACTION_RUNNING.load(Ordering::SeqCst)
}

/// Command to execute in the terminal
#[derive(Clone)]
pub struct TerminalCommand {
    pub command: String,
    pub args: Vec<String>,
}

impl TerminalCommand {
    /// Create a new terminal command
    pub fn new(command: &str, args: &[&str]) -> Self {
        Self {
            command: command.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// Run multiple terminal commands in sequence with interactive PTY support
///
/// This function runs multiple commands one after another. If any command fails,
/// the sequence stops and reports the error.
///
/// # Arguments
/// * `button` - The button that triggered the action (will be disabled during execution)
/// * `terminal` - The VTE terminal widget for output
/// * `terminal_title` - The label for the terminal title
/// * `commands` - Vector of commands to execute in order
/// * `action_name` - Human-readable name for the action (e.g., "LACT Installation")
///
/// # Example
/// ```no_run
/// run_terminal_commands(
///     &button,
///     &terminal,
///     &title_label,
///     vec![
///         TerminalCommand::new("paru", &["-S", "lact"]),
///         TerminalCommand::new("sudo", &["systemctl", "enable", "--now", "lactd"]),
///     ],
///     "LACT Setup",
/// );
/// ```
pub fn run_terminal_commands(
    button: &Button,
    terminal: &Terminal,
    terminal_title: &Label,
    commands: Vec<TerminalCommand>,
    action_name: &str,
) {
    if commands.is_empty() {
        error!("No commands provided to run_terminal_commands");
        return;
    }

    ACTION_RUNNING.store(true, Ordering::SeqCst);

    info!("Starting action: {} - {} commands to execute", action_name, commands.len());
    terminal_title.set_label(&format!("Terminal Output - {}", action_name));
    button.set_sensitive(false);

    terminal.reset(true, true);

    terminal.feed(format!("=== {} ===\r\n\r\n", action_name).as_bytes());
    terminal.feed(b"The following commands will be executed:\r\n\r\n");

    for (i, cmd) in commands.iter().enumerate() {
        terminal.feed(format!("  {}. {} {}\r\n", i + 1, cmd.command, cmd.args.join(" ")).as_bytes());
    }
    terminal.feed(b"\r\n");
    terminal.feed(b"Do you want to proceed? [Y/n]: ");

    wait_for_confirmation(button, terminal, terminal_title, commands, action_name);
}

/// Wait for user to press Y/y before executing commands
fn wait_for_confirmation(
    button: &Button,
    terminal: &Terminal,
    terminal_title: &Label,
    commands: Vec<TerminalCommand>,
    action_name: &str,
) {
    let button_clone = button.clone();
    let title_clone = terminal_title.clone();
    let action_name_clone = action_name.to_string();

    let handler_id_cell: Rc<RefCell<Option<glib::SignalHandlerId>>> = Rc::new(RefCell::new(None));
    let handler_id_cell_clone = handler_id_cell.clone();
    let input_buffer: Rc<RefCell<String>> = Rc::new(RefCell::new(String::new()));
    let input_buffer_clone = input_buffer.clone();

    let handler_id = terminal.connect_commit(move |term, text, _len| {
        for ch in text.chars() {
            if ch == '\r' || ch == '\n' {
                let input = input_buffer_clone.borrow().trim().to_lowercase();

                if input == "y" || input == "yes" || input.is_empty() {
                    term.feed(b"\r\n\r\n");
                    term.feed(format!("=== Starting {} ===\r\n\r\n", action_name_clone).as_bytes());

                    execute_command_sequence(&button_clone, term, &title_clone, commands.clone(), &action_name_clone, 0);
                } else {
                    term.feed(b"\r\n\r\n");
                    term.feed(b"========================================\r\n");
                    term.feed(b"X Operation cancelled by user\r\n");
                    term.feed(b"========================================\r\n");
                    term.feed(b"\r\n");
                    title_clone.set_label("Terminal Output");
                    button_clone.set_sensitive(true);
                    ACTION_RUNNING.store(false, Ordering::SeqCst);
                }

                if let Some(id) = handler_id_cell_clone.borrow_mut().take() {
                    term.disconnect(id);
                }
                break;
            } else {
                input_buffer_clone.borrow_mut().push(ch);
                term.feed(format!("{}", ch).as_bytes());
            }
        }
    });

    *handler_id_cell.borrow_mut() = Some(handler_id);
}



/// Internal function to execute commands in sequence
fn execute_command_sequence(
    button: &Button,
    terminal: &Terminal,
    terminal_title: &Label,
    commands: Vec<TerminalCommand>,
    action_name: &str,
    current_index: usize,
) {
    if current_index >= commands.len() {
        info!("{} - All commands completed successfully", action_name);
        terminal.feed(b"\r\n");
        terminal.feed(b"========================================\r\n");
        terminal.feed(format!("✓ {} completed successfully\r\n", action_name).as_bytes());
        terminal.feed(b"========================================\r\n");
        terminal.feed(b"\r\n");
        terminal_title.set_label("Terminal Output");
        button.set_sensitive(true);
        ACTION_RUNNING.store(false, Ordering::SeqCst);
        return;
    }

    let cmd = &commands[current_index];
    info!("Executing command {}/{}: {} {:?}", current_index + 1, commands.len(), cmd.command, cmd.args);

    terminal.feed(format!("\r\n>>> Executing step {}/{}: {} {}\r\n\r\n",
        current_index + 1, commands.len(), cmd.command, cmd.args.join(" ")).as_bytes());

    let pty = match vte4::Pty::new_sync(vte4::PtyFlags::DEFAULT, gio::Cancellable::NONE) {
        Ok(pty) => pty,
        Err(e) => {
            let msg = format!("Failed to create PTY: {}", e);
            error!("{}", msg);
            terminal.feed(format!("ERROR: {}\r\n", msg).as_bytes());
            button.set_sensitive(true);
            terminal_title.set_label("Terminal Output");
            ACTION_RUNNING.store(false, Ordering::SeqCst);
            return;
        }
    };

    terminal.set_pty(Some(&pty));

    CHILD_EXITED_HANDLER.with(|handler| {
        if let Some(old_handler_id) = handler.borrow_mut().take() {
            terminal.disconnect(old_handler_id);
        }
    });

    let button_clone = button.clone();
    let terminal_clone = terminal.clone();
    let title_clone = terminal_title.clone();
    let action_name_clone = action_name.to_string();
    let commands_clone = commands.clone();
    let cmd_clone = cmd.clone();

    let handler_id = terminal.connect_child_exited(move |term, status| {
        info!("Command exited with status: {}", status);

        if status == 0 {
            execute_command_sequence(
                &button_clone,
                term,
                &title_clone,
                commands_clone.clone(),
                &action_name_clone,
                current_index + 1,
            );
        } else {
            let msg = format!("{} {} exited with code {}", cmd_clone.command, cmd_clone.args.join(" "), status);
            error!("{}", msg);
            terminal_clone.feed(b"\r\n");
            terminal_clone.feed(b"========================================\r\n");
            terminal_clone.feed(format!("✗ {}\r\n", msg).as_bytes());
            terminal_clone.feed(b"========================================\r\n");
            terminal_clone.feed(b"\r\n");
            terminal_clone.feed(b"The process encountered an error.\r\n");
            terminal_clone.feed(b"Sequence aborted.\r\n");
            title_clone.set_label("Terminal Output");
            button_clone.set_sensitive(true);
            ACTION_RUNNING.store(false, Ordering::SeqCst);
        }
    });

    CHILD_EXITED_HANDLER.with(|handler| {
        *handler.borrow_mut() = Some(handler_id);
    });

    let button_clone2 = button.clone();
    let title_clone2 = terminal_title.clone();
    let terminal_clone2 = terminal.clone();
    let terminal_watch = terminal.clone();
    let cmd_str = cmd.command.clone();

    let mut spawn_args = vec![cmd.command.as_str()];
    let args_refs: Vec<&str> = cmd.args.iter().map(|s| s.as_str()).collect();
    spawn_args.extend(args_refs);

    pty.spawn_async(
        Some(&std::env::current_dir().unwrap().to_string_lossy()),
        &spawn_args,
        &[],
        glib::SpawnFlags::DEFAULT,
        || {},
        -1,
        gio::Cancellable::NONE,
        move |result| {
            match result {
                Ok(pid) => {
                    info!("Command spawned successfully with PID: {:?}", pid);
                    terminal_watch.watch_child(pid);
                }
                Err(e) => {
                    error!("Failed to spawn command: {}", e);
                    terminal_clone2.feed(format!("\r\nERROR: Failed to spawn {}: {}\r\n", cmd_str, e).as_bytes());
                    button_clone2.set_sensitive(true);
                    title_clone2.set_label("Terminal Output");
                    ACTION_RUNNING.store(false, Ordering::SeqCst);
                }
            }
        },
    );
}


